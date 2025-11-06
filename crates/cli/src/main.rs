use clap::{Parser, Subcommand};
use std::io::{self, Read};
use std::fs;
use std::path::PathBuf;
use weightlifting_core::{Plan, AppPaths, ExportStager, VersionedPlan, PlanVersion, VersionState, VersionMetadata};
use weightlifting_core::{BuiltinCharts, VolumeMetric, PRDisplayMode};
use weightlifting_indexer::cache::MetricsCache;
use weightlifting_validate::PlanValidator;
use chrono::NaiveDate;

/// **Death to Windows!** - Weightlifting Desktop CLI (Linux native)
#[derive(Parser)]
#[command(name = "comp")]
#[command(about = "Weightlifting plan management CLI")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Plan management commands
    Plans {
        #[command(subcommand)]
        action: PlanAction,
    },
    /// Chart and stats commands
    Chart {
        #[command(subcommand)]
        action: ChartAction,
    },
}

#[derive(Subcommand)]
enum PlanAction {
    /// Validate a plan from JSON input
    Validate {
        /// Read plan from stdin instead of file
        #[arg(long)]
        r#in: bool,
        /// Plan file path (if not using stdin)
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Save a plan from JSON input
    Save {
        /// Read plan from stdin instead of file
        #[arg(long)]
        r#in: bool,
        /// Plan file path (if not using stdin)
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Get a plan by ID
    Get {
        /// Plan ID to retrieve
        #[arg(long)]
        id: String,
        /// Version to retrieve (defaults to latest draft)
        #[arg(long)]
        version: Option<String>,
    },
    /// Export plans with manifest and conflict detection
    Export {
        /// Plan ID to export
        #[arg(long)]
        id: String,
        /// Version to export (defaults to latest draft)
        #[arg(long)]
        version: Option<String>,
        /// Mount point for export staging
        #[arg(long)]
        mount: PathBuf,
        /// Show what would be done without writing files
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum ChartAction {
    /// Generate Vega-Lite chart specification
    EmitSpec {
        /// Chart type to generate
        #[arg(long)]
        chart_type: String,
        /// Exercise filter (for e1rm and pr charts)
        #[arg(long)]
        exercise: Option<String>,
        /// Start date filter (YYYY-MM-DD)
        #[arg(long)]
        start_date: Option<String>,
        /// End date filter (YYYY-MM-DD)
        #[arg(long)]
        end_date: Option<String>,
        /// Output file (defaults to stdout)
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Render chart to file format
    Render {
        /// Chart specification file (JSON)
        #[arg(long)]
        spec: PathBuf,
        /// Output format
        #[arg(long)]
        format: String, // svg, png, pdf
        /// Output file
        #[arg(long)]
        output: PathBuf,
    },
    /// Export chart data as CSV
    ExportCsv {
        /// Chart type to export data for
        #[arg(long)]
        chart_type: String,
        /// Exercise filter (for e1rm and pr charts)
        #[arg(long)]
        exercise: Option<String>,
        /// Start date filter (YYYY-MM-DD)
        #[arg(long)]
        start_date: Option<String>,
        /// End date filter (YYYY-MM-DD)
        #[arg(long)]
        end_date: Option<String>,
        /// Output file
        #[arg(long)]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    let paths = match AppPaths::new() {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("Failed to initialize app paths: {}", e);
            std::process::exit(4);
        }
    };
    
    match cli.command {
        Commands::Plans { action } => {
            match handle_plan_command(action, &paths).await {
                Ok(()) => {},
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(2);
                }
            }
        },
        Commands::Chart { action } => {
            match handle_chart_command(action, &paths).await {
                Ok(()) => {},
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(2);
                }
            }
        },
    }
    
    Ok(())
}

async fn handle_plan_command(action: PlanAction, paths: &AppPaths) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        PlanAction::Validate { r#in, file } => {
            let plan_json = if r#in {
                let mut input = String::new();
                io::stdin().read_to_string(&mut input)?;
                input
            } else if let Some(file_path) = file {
                fs::read_to_string(file_path)?
            } else {
                return Err("Must specify either --in flag or --file path".into());
            };

            let plan: Plan = serde_json::from_str(&plan_json)?;
            let validator = PlanValidator::new()?;
            let result = validator.validate(&plan);

            // Output JSON result to stdout
            println!("{}", serde_json::to_string_pretty(&result)?);

            // Human-readable summary to stderr
            eprintln!("Validation summary: {} errors, {} warnings", 
                      result.errors.len(), 
                      result.warnings.len());

            // Exit with appropriate code
            if !result.errors.is_empty() {
                std::process::exit(2);
            }
        },
        PlanAction::Save { r#in, file } => {
            let plan_json = if r#in {
                let mut input = String::new();
                io::stdin().read_to_string(&mut input)?;
                input
            } else if let Some(file_path) = file {
                fs::read_to_string(file_path)?
            } else {
                return Err("Must specify either --in flag or --file path".into());
            };

            let plan: Plan = serde_json::from_str(&plan_json)?;
            
            // Generate plan ID if not present - using plan name as base
            let plan_id = generate_plan_id(&plan.name);
            
            // Save as draft
            let draft_path = paths.draft_path(&plan_id);
            fs::create_dir_all(draft_path.parent().unwrap())?;
            fs::write(&draft_path, &plan_json)?;

            let response = serde_json::json!({
                "plan_id": plan_id,
                "version": "draft",
                "status": "draft",
                "path": draft_path.to_string_lossy()
            });

            println!("{}", serde_json::to_string_pretty(&response)?);
        },
        PlanAction::Get { id, version } => {
            let plan_path = if let Some(v) = version {
                paths.active_plan_path(&id, &v)
            } else {
                paths.draft_path(&id)
            };

            if !plan_path.exists() {
                eprintln!("Plan not found: {}", id);
                std::process::exit(4);
            }

            let plan_json = fs::read_to_string(plan_path)?;
            println!("{}", plan_json);
        },
        PlanAction::Export { id, version, mount, dry_run } => {
            // Load plan
            let plan_path = if let Some(ref v) = version {
                paths.active_plan_path(&id, v)
            } else {
                paths.draft_path(&id)
            };

            if !plan_path.exists() {
                eprintln!("Plan not found: {}", id);
                std::process::exit(4);
            }

            let plan_json = fs::read_to_string(&plan_path)?;
            let plan: Plan = serde_json::from_str(&plan_json)?;
            
            // Create versioned plan wrapper
            let plan_version = if let Some(ref v) = version {
                parse_version(v)?
            } else {
                PlanVersion::new(1, 0, 0)
            };
            
            let versioned_plan = VersionedPlan {
                plan,
                version: plan_version,
                state: VersionState::Draft,
                metadata: VersionMetadata {
                    created_at: chrono::Utc::now().to_rfc3339(),
                    author: None,
                    message: None,
                    tags: vec!["cli-export".to_string()],
                    parent_version: None,
                },
            };

            // Stage export
            let mut stager = ExportStager::new();
            stager.add_plan(id.clone(), versioned_plan)?;
            stager.analyze();

            // Check for conflicts
            let conflicts = stager.get_conflicts();
            if !conflicts.is_empty() {
                eprintln!("Export conflicts detected:");
                for conflict in conflicts {
                    eprintln!("  • {:?} ({:?}): {}", 
                             conflict.conflict_type, 
                             conflict.severity, 
                             conflict.description);
                }
                
                // Check for existing files at mount point
                let export_path = mount.join("plans").join(&id);
                if export_path.exists() {
                    eprintln!("  • File exists at mount point: {}", export_path.display());
                }
                
                if !stager.can_export() {
                    std::process::exit(3); // Exit code 3 for conflicts
                }
            }

            // Generate manifest
            let manifest = stager.generate_manifest(
                Some("CLI Export".to_string()),
                Some(format!("Export of plan {} via CLI", id))
            );

            // Output manifest JSON to stdout
            println!("{}", serde_json::to_string_pretty(&manifest)?);

            if !dry_run {
                // Perform actual export
                let export_dir = mount.join("plans").join(&id);
                fs::create_dir_all(&export_dir)?;
                
                for plan_info in &manifest.plans {
                    let export_path = mount.join(&plan_info.path);
                    if let Some(parent) = export_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    // Re-serialize plan for export
                    if let Some(versioned_plan) = stager.get_plans().get(&plan_info.id) {
                        let export_json = serde_json::to_string_pretty(&versioned_plan.plan)?;
                        fs::write(&export_path, export_json)?;
                    }
                }
                
                eprintln!("Export completed to: {}", mount.display());
            } else {
                eprintln!("Dry run - no files written");
            }
        },
    }
    
    Ok(())
}

fn generate_plan_id(name: &str) -> String {
    // Convert name to valid ID (alphanumeric + underscores)
    name.chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

fn parse_version(version_str: &str) -> Result<PlanVersion, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = version_str.split('.').collect();
    if parts.len() != 3 {
        return Err(format!("Invalid version format: {}. Expected major.minor.patch", version_str).into());
    }
    
    let major = parts[0].parse::<u32>()?;
    let minor = parts[1].parse::<u32>()?;
    let patch = parts[2].parse::<u32>()?;
    
    Ok(PlanVersion::new(major, minor, patch))
}

async fn handle_chart_command(action: ChartAction, paths: &AppPaths) -> Result<(), Box<dyn std::error::Error>> {
    let cache = MetricsCache::new(paths);
    
    match action {
        ChartAction::EmitSpec { chart_type, exercise, start_date, end_date, output } => {
            let date_range = parse_date_range(start_date.as_deref(), end_date.as_deref())?;
            
            let spec = match chart_type.as_str() {
                "e1rm" => {
                    let e1rm_data = cache.load_e1rm_data()?;
                    let filtered_data = filter_e1rm_data(&e1rm_data, exercise.as_deref(), date_range);
                    BuiltinCharts::e1rm_over_time(&filtered_data, exercise.as_deref(), None)
                },
                "volume" => {
                    let volume_data = cache.load_volume_data()?;
                    let filtered_data = filter_volume_data(&volume_data, date_range);
                    BuiltinCharts::weekly_volume_by_bodypart(&filtered_data, None, VolumeMetric::TonnageKg, None)
                },
                "pr" => {
                    let pr_data = cache.load_pr_data()?;
                    let filtered_data = filter_pr_data(&pr_data, exercise.as_deref(), date_range);
                    BuiltinCharts::pr_board(&filtered_data, exercise.as_deref(), PRDisplayMode::Bar, None)
                },
                "heatmap" => {
                    let e1rm_data = cache.load_e1rm_data()?;
                    let session_dates: Vec<NaiveDate> = e1rm_data.iter()
                        .map(|d| d.date)
                        .collect();
                    let (start, end) = date_range.unwrap_or_else(|| {
                        let min_date = session_dates.iter().min().copied().unwrap_or_else(|| NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
                        let max_date = session_dates.iter().max().copied().unwrap_or_else(|| NaiveDate::from_ymd_opt(2025, 12, 31).unwrap());
                        (min_date, max_date)
                    });
                    BuiltinCharts::session_frequency_heatmap(&session_dates, start, end, None)
                },
                _ => {
                    return Err(format!("Unknown chart type: {}. Available: e1rm, volume, pr, heatmap", chart_type).into());
                }
            };
            
            let json_spec = spec.to_json_string()?;
            
            if let Some(output_path) = output {
                fs::write(output_path, json_spec)?;
            } else {
                println!("{}", json_spec);
            }
        },
        ChartAction::Render { spec: _spec, format: _format, output: _output } => {
            eprintln!("Chart rendering not yet implemented - requires headless browser integration");
            std::process::exit(1);
        },
        ChartAction::ExportCsv { chart_type, exercise, start_date, end_date, output } => {
            let date_range = parse_date_range(start_date.as_deref(), end_date.as_deref())?;
            
            match chart_type.as_str() {
                "e1rm" => {
                    let e1rm_data = cache.load_e1rm_data()?;
                    let filtered_data = filter_e1rm_data(&e1rm_data, exercise.as_deref(), date_range);
                    export_e1rm_csv(&filtered_data, &output)?;
                },
                "volume" => {
                    let volume_data = cache.load_volume_data()?;
                    let filtered_data = filter_volume_data(&volume_data, date_range);
                    export_volume_csv(&filtered_data, &output)?;
                },
                "pr" => {
                    let pr_data = cache.load_pr_data()?;
                    let filtered_data = filter_pr_data(&pr_data, exercise.as_deref(), date_range);
                    export_pr_csv(&filtered_data, &output)?;
                },
                _ => {
                    return Err(format!("Unknown chart type for CSV export: {}. Available: e1rm, volume, pr", chart_type).into());
                }
            }
            
            eprintln!("CSV data exported to: {}", output.display());
        },
    }
    
    Ok(())
}

fn parse_date_range(start: Option<&str>, end: Option<&str>) -> Result<Option<(NaiveDate, NaiveDate)>, Box<dyn std::error::Error>> {
    match (start, end) {
        (Some(s), Some(e)) => {
            let start_date = NaiveDate::parse_from_str(s, "%Y-%m-%d")?;
            let end_date = NaiveDate::parse_from_str(e, "%Y-%m-%d")?;
            Ok(Some((start_date, end_date)))
        },
        (Some(s), None) => {
            let start_date = NaiveDate::parse_from_str(s, "%Y-%m-%d")?;
            let end_date = chrono::Local::now().date_naive();
            Ok(Some((start_date, end_date)))
        },
        (None, Some(e)) => {
            let end_date = NaiveDate::parse_from_str(e, "%Y-%m-%d")?;
            let start_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(); // Far past
            Ok(Some((start_date, end_date)))
        },
        (None, None) => Ok(None),
    }
}

fn filter_e1rm_data(
    data: &[weightlifting_indexer::metrics::E1RMDataPoint], 
    exercise: Option<&str>,
    date_range: Option<(NaiveDate, NaiveDate)>
) -> Vec<(String, NaiveDate, f64)> {
    data.iter()
        .filter(|d| exercise.is_none_or(|ex| d.exercise == ex))
        .filter(|d| date_range.is_none_or(|(start, end)| d.date >= start && d.date <= end))
        .map(|d| (d.exercise.clone(), d.date, d.e1rm_kg))
        .collect()
}

fn filter_volume_data(
    data: &[weightlifting_indexer::metrics::VolumeDataPoint],
    date_range: Option<(NaiveDate, NaiveDate)>
) -> Vec<(String, NaiveDate, u32, u32, f64)> {
    data.iter()
        .filter(|d| date_range.is_none_or(|(start, end)| d.week_start >= start && d.week_start <= end))
        .map(|d| (d.category.clone(), d.week_start, d.total_sets, d.total_reps, d.total_tonnage_kg))
        .collect()
}

fn filter_pr_data(
    data: &[weightlifting_indexer::metrics::PRDataPoint],
    exercise: Option<&str>,
    date_range: Option<(NaiveDate, NaiveDate)>
) -> Vec<(String, String, NaiveDate, f64, Option<u32>)> {
    data.iter()
        .filter(|d| exercise.is_none_or(|ex| d.exercise == ex))
        .filter(|d| date_range.is_none_or(|(start, end)| d.date >= start && d.date <= end))
        .map(|d| (d.exercise.clone(), d.pr_type.clone(), d.date, d.value, d.reps))
        .collect()
}

fn export_e1rm_csv(data: &[(String, NaiveDate, f64)], output: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(output)?;
    
    // Write header
    wtr.write_record(["exercise", "date", "e1rm_kg", "e1rm_lb"])?;
    
    // Write data
    for (exercise, date, e1rm_kg) in data {
        let e1rm_lb = e1rm_kg * 2.20462;
        wtr.write_record([
            exercise.as_str(),
            &date.format("%Y-%m-%d").to_string(),
            &e1rm_kg.to_string(),
            &e1rm_lb.to_string(),
        ])?;
    }
    
    wtr.flush()?;
    Ok(())
}

fn export_volume_csv(data: &[(String, NaiveDate, u32, u32, f64)], output: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(output)?;
    
    // Write header  
    wtr.write_record(["exercise", "week_start", "total_sets", "total_reps", "tonnage_kg", "tonnage_lb"])?;
    
    // Write data
    for (exercise, week_start, sets, reps, tonnage_kg) in data {
        let tonnage_lb = tonnage_kg * 2.20462;
        wtr.write_record([
            exercise.as_str(),
            &week_start.format("%Y-%m-%d").to_string(),
            &sets.to_string(),
            &reps.to_string(),
            &tonnage_kg.to_string(),
            &tonnage_lb.to_string(),
        ])?;
    }
    
    wtr.flush()?;
    Ok(())
}

fn export_pr_csv(data: &[(String, String, NaiveDate, f64, Option<u32>)], output: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(output)?;
    
    // Write header
    wtr.write_record(["exercise", "pr_type", "date", "weight_kg", "weight_lb", "reps"])?;
    
    // Write data
    for (exercise, pr_type, date, weight_kg, reps) in data {
        let weight_lb = weight_kg * 2.20462;
        wtr.write_record([
            exercise.as_str(),
            pr_type.as_str(),
            &date.format("%Y-%m-%d").to_string(),
            &weight_kg.to_string(),
            &weight_lb.to_string(),
            &reps.map_or("".to_string(), |r| r.to_string()),
        ])?;
    }
    
    wtr.flush()?;
    Ok(())
}
