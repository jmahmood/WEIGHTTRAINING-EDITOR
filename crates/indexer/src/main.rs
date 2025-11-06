use clap::{Parser, Subcommand};
use std::path::PathBuf;
use weightlifting_core::AppPaths;

pub mod csv_parser;
pub mod metrics;
pub mod cache;

use csv_parser::SessionCsvParser;
use metrics::{E1RMCalculator, VolumeCalculator, PRTracker};
use cache::MetricsCache;

/// Weightlifting data indexer for generating cached metrics from session CSV data
#[derive(Parser)]
#[command(name = "weightlifting-indexer")]
#[command(about = "Generate cached metrics from session CSV data")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Process CSV files and generate cached metrics
    Process {
        /// Path to session CSV file or directory
        #[arg(long)]
        input: PathBuf,
        /// Force re-processing even if cache is up to date
        #[arg(long)]
        force: bool,
    },
    /// Clear all cached metrics
    Clear,
    /// Show cache status and statistics
    Status,
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

    let cache = MetricsCache::new(&paths);
    
    match cli.command {
        Commands::Process { input, force } => {
            process_csv_data(&input, &cache, force).await?;
        },
        Commands::Clear => {
            cache.clear_all()?;
            println!("All cached metrics cleared");
        },
        Commands::Status => {
            show_cache_status(&cache)?;
        },
    }
    
    Ok(())
}

async fn process_csv_data(
    input: &PathBuf, 
    cache: &MetricsCache, 
    _force: bool
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing CSV data from: {}", input.display());
    
    let parser = SessionCsvParser::new();
    let sessions = parser.parse_csv_file(input)?;
    
    println!("Parsed {} session records", sessions.len());
    
    // Calculate metrics
    let e1rm_calc = E1RMCalculator::new();
    let volume_calc = VolumeCalculator::new();
    let pr_tracker = PRTracker::new();
    
    // Process E1RM data
    let e1rm_data = e1rm_calc.calculate_historical_e1rms(&sessions)?;
    cache.store_e1rm_data(&e1rm_data)?;
    
    // Process volume data
    let volume_data = volume_calc.calculate_weekly_volumes(&sessions)?;
    cache.store_volume_data(&volume_data)?;
    
    // Process PR data
    let pr_data = pr_tracker.identify_prs(&sessions)?;
    cache.store_pr_data(&pr_data)?;
    
    println!("Metrics processing completed and cached");
    Ok(())
}

fn show_cache_status(cache: &MetricsCache) -> Result<(), Box<dyn std::error::Error>> {
    let status = cache.get_status()?;
    
    println!("Cache Status:");
    println!("  Location: {}", cache.cache_dir().display());
    println!("  E1RM entries: {}", status.e1rm_entries);
    println!("  Volume entries: {}", status.volume_entries);
    println!("  PR entries: {}", status.pr_entries);
    println!("  Last updated: {}", status.last_updated.unwrap_or("Never".to_string()));
    
    Ok(())
}
