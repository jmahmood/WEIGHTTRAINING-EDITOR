use gtk4::{Dialog, DialogFlags, ResponseType, Box as GtkBox, Orientation, Label, Entry, Button, DropDown, StringList, ToggleButton, Separator, FileChooserAction, FileChooserDialog};
use gtk4::prelude::*;
use glib::clone;
use glib::ControlFlow;
use std::path::{Path, PathBuf};
use std::process::Command;
// use std::io; // not currently used
use serde::Deserialize;
use weightlifting_core::AppPaths;

use crate::ui::util::{parent_for_dialog, standardize_dialog, bind_ctrl_enter_to_accept};
use crate::state::AppState;

use std::sync::{Arc, Mutex};

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct SyncJsonOk {
    status: String,
    transport: Option<String>,
    files: Option<Vec<TransferredFile>>, 
    bytes: Option<u64>,
    duration_ms: Option<u64>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct TransferredFile {
    name: String,
    bytes: Option<u64>,
    sha256: Option<String>,
    action: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct DiscoverJsonOk {
    status: String,
    dirs: Vec<DiscoverDir>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
struct DiscoverDir {
    path: String,
    free_bytes: Option<u64>,
    owner: Option<String>,
    writable: Option<bool>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[allow(dead_code)]
struct ScriptErrorJson {
    status: Option<String>,
    code: Option<String>,
    message: Option<String>,
    log_path: Option<String>,
    duration_ms: Option<u64>,
}

enum DiscoverOutcome {
    Ok(Vec<String>),
    Err(ScriptErrorJson),
}

fn resolve_sync_script(paths: &AppPaths) -> Option<PathBuf> {
    if let Ok(p) = std::env::var("WEIGHTLIFTING_SYNC_SCRIPT") { return Some(PathBuf::from(p)); }
    // Try relative to CWD
    let cwd = std::env::current_dir().ok();
    if let Some(dir) = cwd {
        let cand = dir.join("scripts/sync.sh");
        if cand.exists() { return Some(cand); }
    }
    // Try ancestors of current_exe()
    if let Ok(mut exe) = std::env::current_exe() {
        for _ in 0..5 {
            if let Some(parent) = exe.parent() {
                let cand = parent.join("scripts/sync.sh");
                if cand.exists() { return Some(cand); }
                exe = parent.to_path_buf();
            }
        }
    }
    // Fallback: auto-install embedded script under data_dir
    ensure_installed_sync_script(paths).ok()
}

fn ensure_installed_sync_script(paths: &AppPaths) -> Result<PathBuf, String> {
    // Embed script at compile time
    const SYNC_SH: &str = include_str!("../../../../scripts/sync.sh");
    let target_dir = paths.data_dir.join("scripts");
    std::fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
    let target = target_dir.join("sync.sh");
    // Write or overwrite if content differs
    let write_needed = match std::fs::read_to_string(&target) {
        Ok(existing) => existing != SYNC_SH,
        Err(_) => true,
    };
    if write_needed {
        std::fs::write(&target, SYNC_SH).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perm = std::fs::metadata(&target).map_err(|e| e.to_string())?.permissions();
            perm.set_mode(0o755);
            std::fs::set_permissions(&target, perm).map_err(|e| e.to_string())?;
        }
    }
    Ok(target)
}

fn default_local_sync_root(paths: &AppPaths) -> PathBuf {
    paths.state_dir.join("sync")
}

pub fn show_send_to_device_dialog(state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>) {

    let dlg = Dialog::with_buttons(
        Some("Send Plan to Device"),
        parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Send", ResponseType::Accept)]
    );
    standardize_dialog(&dlg);

    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_top(16).margin_bottom(16).margin_start(20).margin_end(20)
        .spacing(10)
        .build();

    // Save reminder
    let save_hint = Label::builder()
        .label("Please save your plan before sending to ensure the latest version is transferred.")
        .wrap(true).build();
    content.append(&save_hint);

    // Local root chooser
    let local_row = GtkBox::builder().orientation(Orientation::Horizontal).spacing(6).build();
    let local_lbl = Label::builder().label("Local sync root").halign(gtk4::Align::Start).build();
    let local_entry = Entry::new();
    local_entry.set_hexpand(true);
    // Prefill from preferences
    let (pref_transport, pref_local_root, pref_host, pref_port, pref_ssh_root, pref_usb_mount, pref_usb_root) = {
        let s = state.lock().unwrap();
        let p = &s.preferences;
        (
            p.last_sync_transport.clone().unwrap_or_else(|| "ssh".to_string()),
            p.last_sync_local_root.clone(),
            p.last_sync_ssh_host.clone(),
            p.last_sync_ssh_port.clone(),
            p.last_sync_ssh_remote_root.clone(),
            p.last_sync_usb_mount.clone(),
            p.last_sync_usb_remote_root.clone(),
        )
    };
    let initial_local = pref_local_root.unwrap_or_else(|| default_local_sync_root(&paths).to_string_lossy().to_string());
    local_entry.set_text(&initial_local);
    let local_btn = Button::with_label("Browse…");
    local_row.append(&local_lbl); local_row.append(&local_entry); local_row.append(&local_btn);
    content.append(&local_row);

    // Transport selectors
    let transport_row = GtkBox::builder().orientation(Orientation::Horizontal).spacing(12).build();
    let ssh_toggle = ToggleButton::with_label("SSH"); ssh_toggle.set_active(true);
    let usb_toggle = ToggleButton::with_label("USB");
    let tlabel = Label::builder().label("Transport").halign(gtk4::Align::Start).build();
    transport_row.append(&tlabel); transport_row.append(&ssh_toggle); transport_row.append(&usb_toggle);
    content.append(&transport_row);

    let sep = Separator::new(Orientation::Horizontal); content.append(&sep);

    // SSH fields
    let ssh_box = GtkBox::builder().orientation(Orientation::Vertical).spacing(8).build();
    let ssh_host_row = GtkBox::builder().orientation(Orientation::Horizontal).spacing(6).build();
    let ssh_host_lbl = Label::builder().label("Host (user@host)").halign(gtk4::Align::Start).build();
    let ssh_host_entry = Entry::new(); ssh_host_entry.set_hexpand(true); ssh_host_entry.set_placeholder_text(Some("sync@device.local"));
    if let Some(h) = pref_host { ssh_host_entry.set_text(&h); }
    let ssh_port_entry = Entry::new(); ssh_port_entry.set_width_chars(6); ssh_port_entry.set_placeholder_text(Some("22"));
    if let Some(p) = pref_port { ssh_port_entry.set_text(&p); }
    ssh_host_row.append(&ssh_host_lbl); ssh_host_row.append(&ssh_host_entry); ssh_host_row.append(&ssh_port_entry);
    ssh_box.append(&ssh_host_row);

    let ssh_root_row = GtkBox::builder().orientation(Orientation::Horizontal).spacing(6).build();
    let ssh_root_lbl = Label::builder().label("Remote root").halign(gtk4::Align::Start).build();
    let ssh_root_entry = Entry::new(); ssh_root_entry.set_hexpand(true); ssh_root_entry.set_placeholder_text(Some("/home/sync/plans"));
    if let Some(r) = pref_ssh_root { ssh_root_entry.set_text(&r); }
    let discover_btn = Button::with_label("Discover…");
    ssh_root_row.append(&ssh_root_lbl); ssh_root_row.append(&ssh_root_entry); ssh_root_row.append(&discover_btn);
    ssh_box.append(&ssh_root_row);

    let discovered_list = StringList::new(&[]);
    let discovered_dropdown = DropDown::new(Some(discovered_list.clone()), None::<gtk4::Expression>);
    discovered_dropdown.set_hexpand(true);
    ssh_box.append(&discovered_dropdown);

    // When user picks a discovered path, update the root entry
    discovered_dropdown.connect_selected_notify(clone!(@weak discovered_list, @weak ssh_root_entry => move |dd| {
        let idx = dd.selected();
        if idx < discovered_list.n_items() {
            if let Some(item) = discovered_list.string(idx) {
                ssh_root_entry.set_text(item.as_str());
            }
        }
    }));

    // USB fields
    let usb_box = GtkBox::builder().orientation(Orientation::Vertical).spacing(8).build();
    let usb_mount_row = GtkBox::builder().orientation(Orientation::Horizontal).spacing(6).build();
    let usb_mount_lbl = Label::builder().label("USB mount").halign(gtk4::Align::Start).build();
    let usb_mount_entry = Entry::new(); usb_mount_entry.set_hexpand(true); usb_mount_entry.set_placeholder_text(Some("/run/media/user/DEVICE"));
    if let Some(m) = pref_usb_mount { usb_mount_entry.set_text(&m); }
    let usb_browse_btn = Button::with_label("Browse…");
    usb_mount_row.append(&usb_mount_lbl); usb_mount_row.append(&usb_mount_entry); usb_mount_row.append(&usb_browse_btn);
    usb_box.append(&usb_mount_row);

    let usb_root_row = GtkBox::builder().orientation(Orientation::Horizontal).spacing(6).build();
    let usb_root_lbl = Label::builder().label("Remote root under mount").halign(gtk4::Align::Start).build();
    let usb_root_entry = Entry::new(); usb_root_entry.set_hexpand(true); usb_root_entry.set_placeholder_text(Some("/plans"));
    if let Some(r) = pref_usb_root { usb_root_entry.set_text(&r); }
    usb_root_row.append(&usb_root_lbl); usb_root_row.append(&usb_root_entry);
    usb_box.append(&usb_root_row);

    // Transport initial selection from prefs
    if pref_transport == "ssh" { ssh_toggle.set_active(true); usb_toggle.set_active(false); } else { ssh_toggle.set_active(false); usb_toggle.set_active(true); }

    content.append(&ssh_box);
    content.append(&usb_box);

    // Toggle visibility
    let update_transport_ui = clone!(@weak ssh_box, @weak usb_box, @weak ssh_toggle, @weak usb_toggle => move || {
        let ssh_on = ssh_toggle.is_active();
        ssh_box.set_visible(ssh_on);
        usb_box.set_visible(!ssh_on);
        if ssh_on && !usb_toggle.is_active() { /* keep opposite */ };
    });
    ssh_toggle.connect_toggled(clone!(@strong update_transport_ui, @weak usb_toggle => move |btn| {
        if btn.is_active() { usb_toggle.set_active(false); }
        update_transport_ui();
    }));
    usb_toggle.connect_toggled(clone!(@strong update_transport_ui, @weak ssh_toggle => move |btn| {
        if btn.is_active() { ssh_toggle.set_active(false); }
        update_transport_ui();
    }));
    update_transport_ui();

    // Browse local root
    local_btn.connect_clicked(clone!(@weak local_entry => move |_| {
        // Open file chooser even if there is no active parent window
        let fc = FileChooserDialog::new(Some("Select Local Sync Root"), parent_for_dialog().as_ref(), FileChooserAction::SelectFolder, &[]);
        fc.add_buttons(&[("Cancel", ResponseType::Cancel), ("Select", ResponseType::Accept)]);
        standardize_dialog(&fc);
        fc.connect_response(clone!(@weak local_entry, @weak fc => move |d, resp| {
            if resp == ResponseType::Accept {
                if let Some(f) = fc.file() { if let Some(p) = f.path() { local_entry.set_text(p.to_string_lossy().as_ref()); } }
            }
            d.close();
        }));
        fc.present();
    }));

    // Browse USB mount
    usb_browse_btn.connect_clicked(clone!(@weak usb_mount_entry => move |_| {
        let fc = FileChooserDialog::new(Some("Select USB Mount"), parent_for_dialog().as_ref(), FileChooserAction::SelectFolder, &[]);
        fc.add_buttons(&[("Cancel", ResponseType::Cancel), ("Select", ResponseType::Accept)]);
        standardize_dialog(&fc);
        fc.connect_response(clone!(@weak usb_mount_entry, @weak fc => move |d, resp| {
            if resp == ResponseType::Accept {
                if let Some(f) = fc.file() { if let Some(p) = f.path() { usb_mount_entry.set_text(p.to_string_lossy().as_ref()); } }
            }
            d.close();
        }));
        fc.present();
    }));

    // Discover remote dirs
    // Status label under dropdown
    let discover_status = Label::builder().label("").halign(gtk4::Align::Start).build();
    ssh_box.append(&discover_status);

    discover_btn.connect_clicked(clone!(@weak discover_btn, @weak ssh_host_entry, @weak ssh_port_entry, @weak local_entry, @weak discovered_list, @weak discover_status, @strong paths => move |_| {
        let mut host = ssh_host_entry.text().to_string();
        if !host.contains('@') { host = format!("root@{}", host); }
        let port = ssh_port_entry.text().to_string();
        let lroot = local_entry.text().to_string();
        if host.trim().is_empty() { show_error("Please enter SSH host as user@host."); return; }
        if let Some(script) = resolve_sync_script(&paths) {
            let old_label = discover_btn.label().map(|s| s.to_string()).unwrap_or_else(|| "Discover…".to_string());
            discover_btn.set_sensitive(false);
            discover_btn.set_label("Discovering…");
            discover_status.set_text("Running discovery…");
            println!("[Discover] host={} port={} local_root={} script={} ", host, port, lroot, script.display());

            // Run in background thread and poll results from GTK
            let script_path = script.to_string_lossy().to_string();
            let (tx, rx) = std::sync::mpsc::channel::<DiscoverOutcome>();
            let host_for_err = host.clone();
            let port_for_err = port.clone();
            std::thread::spawn(move || {
                let mut cmd = Command::new(&script_path);
                cmd.arg("discover-remote-dirs").arg("--transport").arg("ssh").arg("--local-root").arg(&lroot).arg("--remote-host").arg(&host).arg("--timeout").arg("10").arg("--refresh-cache").arg("1");
                if !port.trim().is_empty() { cmd.arg("--remote-port").arg(&port); }
                let outcome = match cmd.output() {
                    Ok(out) => {
                        if !out.status.success() {
                            // Try to parse JSON error; fallback to plain string
                            let mut stdout_s = String::from_utf8_lossy(&out.stdout).to_string();
                            let stderr_s = String::from_utf8_lossy(&out.stderr).to_string();
                            if stdout_s.trim().is_empty() && !stderr_s.trim().is_empty() {
                                stdout_s = stderr_s;
                            }
                            let cleaned = stdout_s.trim().trim_start_matches("Discovery failed:").trim().to_string();
                            match serde_json::from_str::<ScriptErrorJson>(&cleaned) {
                                Ok(mut e) => {
                                    if e.message.as_deref().unwrap_or("").trim().is_empty() {
                                        e.message = Some("Discovery failed".to_string());
                                    }
                                    DiscoverOutcome::Err(e)
                                }
                                Err(_) => {
                                    DiscoverOutcome::Err(ScriptErrorJson { status: Some("error".into()), code: None, message: Some(format!("Discovery failed: {}", cleaned)), log_path: None, duration_ms: None })
                                }
                            }
                        } else {
                            match serde_json::from_slice::<DiscoverJsonOk>(&out.stdout) {
                                Ok(ok) => DiscoverOutcome::Ok(ok.dirs.into_iter().map(|d| d.path).collect::<Vec<String>>()),
                                Err(e) => DiscoverOutcome::Err(ScriptErrorJson { status: Some("error".into()), code: Some("PARSE_ERROR".into()), message: Some(format!("Parse error: {}", e)), log_path: None, duration_ms: None }),
                            }
                        }
                    }
                    Err(e) => DiscoverOutcome::Err(ScriptErrorJson { status: Some("error".into()), code: Some("EXEC_ERROR".into()), message: Some(format!("Exec error: {}", e)), log_path: None, duration_ms: None }),
                };
                let _ = tx.send(outcome);
            });

            let _sid = glib::timeout_add_local(std::time::Duration::from_millis(100), clone!(@weak discovered_list, @weak discover_btn, @weak discover_status => @default-return ControlFlow::Continue, move || {
                match rx.try_recv() {
                    Ok(DiscoverOutcome::Ok(paths_vec)) => {
                        println!("[Discover] found {} dirs", paths_vec.len());
                        while discovered_list.n_items() > 0 { discovered_list.remove(discovered_list.n_items() - 1); }
                        for p in paths_vec.iter() { discovered_list.append(p); }
                        discover_status.set_text(&format!("Found {} directories", paths_vec.len()));
                        discover_btn.set_label(&old_label);
                        discover_btn.set_sensitive(true);
                        ControlFlow::Break
                    }
                    Ok(DiscoverOutcome::Err(errj)) => {
                        // Show unified, styled error dialog for all discovery errors
                        println!("[Discover] error: code={:?} message={:?}", errj.code, errj.message);
                        discover_status.set_text("Discovery failed");
                        show_discovery_error_dialog(&errj, &host_for_err, Some(port_for_err.as_str()));
                        discover_btn.set_label(&old_label);
                        discover_btn.set_sensitive(true);
                        ControlFlow::Break
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => ControlFlow::Continue,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        discover_status.set_text("Discovery worker disconnected");
                        discover_btn.set_label(&old_label);
                        discover_btn.set_sensitive(true);
                        ControlFlow::Break
                    }
                }
            }));
        } else {
            show_error("sync.sh not found and could not be installed.");
            println!("[Discover] sync.sh not found");
        }
    }));

    dlg.content_area().append(&content);
    bind_ctrl_enter_to_accept(&dlg);

    dlg.connect_response(clone!(@strong state, @strong paths, @weak local_entry,
                                @weak ssh_toggle, @weak usb_toggle,
                                @weak ssh_host_entry, @weak ssh_port_entry, @weak ssh_root_entry,
                                @weak usb_mount_entry, @weak usb_root_entry
                                => move |dialog, resp| {
        if resp == ResponseType::Accept {
            // Warn to save if modified
            if !confirm_saved(state.clone(), paths.clone()) { dialog.close(); return; }
            // Collect inputs
            let local_root = local_entry.text().to_string();
            let transport = if ssh_toggle.is_active() { "ssh" } else { "usb-fs" };
            // Persist last used settings
            {
                let mut s = state.lock().unwrap();
                s.preferences.last_sync_transport = Some(transport.to_string());
                s.preferences.last_sync_local_root = Some(local_root.clone());
                if transport == "ssh" {
                    s.preferences.last_sync_ssh_host = Some(ssh_host_entry.text().to_string());
                    s.preferences.last_sync_ssh_port = Some(ssh_port_entry.text().to_string());
                    s.preferences.last_sync_ssh_remote_root = Some(ssh_root_entry.text().to_string());
                } else {
                    s.preferences.last_sync_usb_mount = Some(usb_mount_entry.text().to_string());
                    s.preferences.last_sync_usb_remote_root = Some(usb_root_entry.text().to_string());
                }
                let _ = s.preferences.save(&paths);
            }

            let result = if transport == "ssh" {
                let host = ssh_host_entry.text().to_string();
                let port = ssh_port_entry.text().to_string();
                let remote_root = ssh_root_entry.text().to_string();
                send_current_plan_via_ssh(state.clone(), paths.clone(), &local_root, &host, if port.trim().is_empty() { None } else { Some(port.as_str()) }, &remote_root)
            } else {
                let mount = usb_mount_entry.text().to_string();
                let remote_root = usb_root_entry.text().to_string();
                send_current_plan_via_usb(state.clone(), paths.clone(), &local_root, &mount, &remote_root)
            };

            match result {
                Ok(msg) => show_info(&msg),
                Err(e) => show_error(&e),
            }
        }
        dialog.close();
    }));

    dlg.present();
}

fn stage_current_plan_to_outbox(state: Arc<Mutex<AppState>>, paths: &AppPaths, local_root: &Path) -> Result<PathBuf, String> {
    let app_state = state.lock().unwrap();
    let plan_id = app_state.plan_id.clone().ok_or_else(|| "No plan open".to_string())?;
    // Prefer explicit file path, else drafts_dir
    let src_path = if let Some(p) = &app_state.current_file_path { p.clone() } else { paths.drafts_dir().join(format!("{}.json", plan_id)) };

    if !src_path.exists() { return Err(format!("Plan file not found at {}. Please save first.", src_path.display())); }
    // Ensure outbox
    let outbox = local_root.join("outbox");
    std::fs::create_dir_all(&outbox).map_err(|e| e.to_string())?;
    let dest = outbox.join(src_path.file_name().unwrap_or_default());
    std::fs::copy(&src_path, &dest).map_err(|e| e.to_string())?;
    Ok(dest)
}

fn run_sync_send(paths: &AppPaths, args: &[&str]) -> Result<SyncJsonOk, String> {
    let script = resolve_sync_script(paths).ok_or_else(|| "sync.sh not found".to_string())?;
    let out = Command::new(&script).args(args).output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        let mut s = String::from_utf8_lossy(&out.stdout).to_string();
        if s.trim().is_empty() {
            let serr = String::from_utf8_lossy(&out.stderr).to_string();
            if !serr.trim().is_empty() { s = serr; }
        }
        return Err(format!("Send failed: {}", s));
    }
    serde_json::from_slice::<SyncJsonOk>(&out.stdout).map_err(|e| format!("Failed to parse script JSON: {}", e))
}

fn send_current_plan_via_ssh(state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>, local_root: &str, host: &str, port: Option<&str>, remote_root: &str) -> Result<String, String> {
    let local_root_pb = PathBuf::from(local_root);
    let _staged = stage_current_plan_to_outbox(state, &paths, &local_root_pb)?;
    let host_arg = if host.contains('@') { host.to_string() } else { format!("root@{}", host) };
    let mut args = vec!["send", "--transport", "ssh", "--local-root", local_root, "--remote-host", host_arg.as_str(), "--remote-root", remote_root];
    if let Some(p) = port { args.push("--remote-port"); args.push(p); }
    let res = run_sync_send(&paths, &args)?;
    let sent_count = res.files.as_ref().map(|v| v.len()).unwrap_or(0);
    Ok(format!("Sent {} file(s) via SSH.", sent_count))
}

fn send_current_plan_via_usb(state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>, local_root: &str, mount: &str, remote_root: &str) -> Result<String, String> {
    let local_root_pb = PathBuf::from(local_root);
    let _staged = stage_current_plan_to_outbox(state, &paths, &local_root_pb)?;
    let args = ["send", "--transport", "usb-fs", "--usb-mount", mount, "--remote-root", remote_root, "--local-root", local_root];
    let res = run_sync_send(&paths, &args)?;
    let sent_count = res.files.as_ref().map(|v| v.len()).unwrap_or(0);
    Ok(format!("Sent {} file(s) via USB.", sent_count))
}

fn confirm_saved(state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>) -> bool {
    // If app state indicates modified, show save warning dialog with choices
    use gtk4::{MessageDialog, ButtonsType, MessageType};
    let is_modified = { state.lock().unwrap().is_modified };
    if !is_modified { return true; }
    let dlg = MessageDialog::new(
        parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        MessageType::Question,
        ButtonsType::None,
        "You have unsaved changes. Save before sending?"
    );
    standardize_dialog(&dlg);
    dlg.add_button("Cancel", ResponseType::Cancel);
    dlg.add_button("Send Without Saving", ResponseType::No);
    dlg.add_button("Save and Send", ResponseType::Yes);

    let decision = std::rc::Rc::new(std::cell::Cell::new(ResponseType::Cancel));
    let done = std::rc::Rc::new(std::cell::Cell::new(false));
    let d1 = decision.clone(); let dn = done.clone();
    dlg.connect_response(move |d, resp| { d1.set(resp); dn.set(true); d.close(); });
    dlg.present();
    let ctx = glib::MainContext::default();
    while !done.get() { ctx.iteration(true); }

    match decision.get() {
        ResponseType::Yes => { // Save
            crate::operations::plan::save_current_plan(state, paths);
            true
        }
        ResponseType::No => true,
        _ => false,
    }
}

fn show_error(message: &str) {
    let d = Dialog::with_buttons(
        Some("Error"), parent_for_dialog().as_ref(), DialogFlags::MODAL, &[("OK", ResponseType::Ok)]
    );
    standardize_dialog(&d);
    let lbl = Label::builder().label(message).wrap(true).build();
    d.content_area().append(&lbl);
    d.connect_response(|d, _| d.close());
    d.present();
}

fn show_info(message: &str) {
    let d = Dialog::with_buttons(
        Some("Info"), parent_for_dialog().as_ref(), DialogFlags::MODAL, &[("OK", ResponseType::Ok)]
    );
    standardize_dialog(&d);
    let lbl = Label::builder().label(message).wrap(true).build();
    d.content_area().append(&lbl);
    d.connect_response(|d, _| d.close());
    d.present();
}

#[allow(dead_code)]
fn show_error_with_log(message: &str, log_path: Option<&str>) {
    let d = Dialog::with_buttons(
        Some("Error"), parent_for_dialog().as_ref(), DialogFlags::MODAL, &[("OK", ResponseType::Ok)]
    );
    standardize_dialog(&d);
    let lbl = Label::builder().label(message).wrap(true).build();
    d.content_area().append(&lbl);
    if let Some(lp) = log_path {
        let btn = Button::with_label("Open Log…");
        let lp_string = lp.to_string();
        btn.connect_clicked(move |_| { open_path_in_default_app(&lp_string); });
        d.content_area().append(&btn);
    }
    d.connect_response(|d, _| d.close());
    d.present();
}

fn open_path_in_default_app(path: &str) {
    let _ = std::process::Command::new("xdg-open").arg(path).spawn();
}

#[allow(dead_code)]
fn format_discovery_error(err: &ScriptErrorJson, host: &str, port: Option<&str>) -> String {
    let code = err.code.as_deref().unwrap_or("");
    match code {
        "SSH_UNREACHABLE" => {
            let mut s = String::new();
            s.push_str("Could not connect to device via SSH. Likely causes: device is powered off, not on the network, or SSH service is not running/listening");
            if let Some(p) = port { if !p.trim().is_empty() { s.push_str(&format!(" on port {}", p)); } }
            s.push('.');
            s.push_str(&format!(" Host: {}.", host));
            if let Some(m) = &err.message { if !m.trim().is_empty() { s.push_str(&format!("\nDetails: {}", m)); } }
            s
        }
        "MISSING_TOOL" => {
            let base = err.message.as_deref().unwrap_or("Required tool missing");
            format!("{} — please install the missing tool on this computer.", base)
        }
        "MISSING_ARG" => {
            let base = err.message.as_deref().unwrap_or("Missing required argument");
            format!("{} — please fill in the required fields.", base)
        }
        "PARSE_ERROR" => {
            let base = err.message.as_deref().unwrap_or("Failed to parse discovery output");
            format!("{}.", base)
        }
        "EXEC_ERROR" => {
            let base = err.message.as_deref().unwrap_or("Failed to run discovery");
            format!("{}.", base)
        }
        _ => {
            err.message.clone().unwrap_or_else(|| "Discovery failed".to_string())
        }
    }
}

fn show_discovery_error_dialog(err: &ScriptErrorJson, host: &str, port: Option<&str>) {
    use gtk4::{Align};
    // Build a more readable, styled error dialog
    let title = match err.code.as_deref().unwrap_or("") {
        "SSH_UNREACHABLE" => "Connection Error",
        "MISSING_TOOL" => "Missing Tool",
        "MISSING_ARG" => "Missing Input",
        "PARSE_ERROR" => "Result Error",
        "EXEC_ERROR" => "Execution Error",
        _ => "Error",
    };

    let d = Dialog::with_buttons(
        Some(title),
        parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("OK", ResponseType::Ok)]
    );
    standardize_dialog(&d);

    // Container with spacing and margins
    let boxv = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(8).margin_bottom(8).margin_start(6).margin_end(6)
        .build();

    // Header (bold)
    let header = Label::new(None);
    header.set_use_markup(true);
    header.set_wrap(true);
    header.set_xalign(0.0);
    let header_text = match err.code.as_deref().unwrap_or("") {
        "SSH_UNREACHABLE" => "<b>Could not connect to device via SSH</b>",
        "MISSING_TOOL" => "<b>Required tool not found</b>",
        "MISSING_ARG" => "<b>Missing required input</b>",
        "PARSE_ERROR" => "<b>Could not read discovery result</b>",
        "EXEC_ERROR" => "<b>Could not run discovery</b>",
        _ => "<b>Discovery failed</b>",
    };
    header.set_markup(header_text);
    boxv.append(&header);

    // Host line (only when meaningful)
    if !host.trim().is_empty() {
        let hp = match port { Some(p) if !p.trim().is_empty() => format!("{}:{}", host, p), _ => host.to_string() };
        let host_lbl = Label::new(None);
        host_lbl.set_use_markup(true);
        host_lbl.set_wrap(true);
        host_lbl.set_xalign(0.0);
        host_lbl.set_markup(&format!("<b>Host:</b> <tt>{}</tt>", glib::markup_escape_text(&hp)));
        boxv.append(&host_lbl);
    }

    // Likely causes / suggested fixes per code
    let bullets = Label::new(None);
    bullets.set_use_markup(true);
    bullets.set_wrap(true);
    bullets.set_xalign(0.0);
    let bullets_markup = match err.code.as_deref().unwrap_or("") {
        "SSH_UNREACHABLE" => "<b>Likely causes:</b>\n• Device is powered off\n• Device is not on the network\n• SSH service is not running/listening",
        "MISSING_TOOL" => "<b>Fix:</b>\n• Install the required tool (e.g., <tt>ssh</tt>)\n• Ensure it is available in <tt>PATH</tt>",
        "MISSING_ARG" => "<b>Fix:</b>\n• Provide all required fields\n• For SSH discovery, enter <tt>user@host</tt> (and optional port)",
        "PARSE_ERROR" => "<b>What you can try:</b>\n• Update the app to the latest version\n• Check the log for script output anomalies",
        "EXEC_ERROR" => "<b>What you can try:</b>\n• Ensure the discovery script is present and executable\n• Check permissions or antivirus/OS policies that block execution",
        _ => "<b>Next steps:</b>\n• Review the details below\n• Open the log for more information",
    };
    bullets.set_markup(bullets_markup);
    boxv.append(&bullets);

    // Details (optional, smaller)
    if let Some(m) = err.message.as_deref() {
        if !m.trim().is_empty() {
            let details = Label::new(None);
            details.set_use_markup(true);
            details.set_wrap(true);
            details.set_xalign(0.0);
            details.set_markup(&format!("<span size='small'><b>Details:</b> {}</span>", glib::markup_escape_text(m)));
            boxv.append(&details);
        }
    }

    d.content_area().append(&boxv);

    // Add Open Log… into the action area so it aligns with OK
    if let Some(lp) = err.log_path.as_deref() {
        let open_btn = d.add_button("Open Log…", ResponseType::Other(1));
        open_btn.set_halign(Align::End);
        let lp_string = lp.to_string();
        d.connect_response(move |dlg, resp| {
            if resp == ResponseType::Other(1) {
                open_path_in_default_app(&lp_string);
            }
            if matches!(resp, ResponseType::Ok | ResponseType::Close | ResponseType::DeleteEvent) {
                dlg.close();
            }
        });
    } else {
        d.connect_response(|dlg, _| dlg.close());
    }

    d.present();
}
