use gtk4::{Dialog, DialogFlags, ResponseType, Box as GtkBox, Label, Entry, CheckButton, FileFilter, FileChooserAction, FileChooserWidget, Orientation};
use gtk4::prelude::*;
use gtk4::gio;
use weightlifting_core::{MediaAttachmentWriter, SetKey, AppPaths};
use std::sync::Arc;

pub fn show_attach_media_dialog(paths: Arc<AppPaths>) {
    let dialog = Dialog::with_buttons(
        Some("Attach Media to Sets"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Apply", ResponseType::Accept)]
    );
    crate::ui::util::standardize_dialog(&dialog);

    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .spacing(12)
        .build();

    let session_label = Label::builder().label("Session ID").halign(gtk4::Align::Start).build();
    let session_entry = Entry::new();
    session_entry.set_placeholder_text(Some("e.g., 2025-08-14T09-35-00Z-001"));

    let ex_label = Label::builder().label("Exercise Code").halign(gtk4::Align::Start).build();
    let ex_entry = Entry::new();
    ex_entry.set_placeholder_text(Some("e.g., BP.BB.FLAT"));

    let setnums_label = Label::builder().label("Set Numbers").halign(gtk4::Align::Start).build();
    let setnums_entry = Entry::new();
    setnums_entry.set_placeholder_text(Some("e.g., 1,2,4-6"));

    let detach_toggle = CheckButton::with_label("Detach instead of attach");

    // File chooser for selecting one or more media files
    let file_chooser = FileChooserWidget::new(FileChooserAction::Open);
    file_chooser.set_select_multiple(true);
    let filter = FileFilter::new();
    filter.add_mime_type("video/*");
    filter.add_mime_type("image/*");
    filter.set_name(Some("Media files (video/image)"));
    file_chooser.add_filter(&filter);

    // Layout
    content.append(&session_label);
    content.append(&session_entry);
    content.append(&ex_label);
    content.append(&ex_entry);
    content.append(&setnums_label);
    content.append(&setnums_entry);
    content.append(&detach_toggle);
    content.append(&file_chooser);
    dialog.content_area().append(&content);
    // Ctrl+Enter triggers Apply
    crate::ui::util::bind_ctrl_enter_to_accept(&dialog);

    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let session_id = session_entry.text().to_string();
            let ex_code = ex_entry.text().to_string();
            let setnums_raw = setnums_entry.text().to_string();
            let do_detach = detach_toggle.is_active();

            if session_id.trim().is_empty() || ex_code.trim().is_empty() || setnums_raw.trim().is_empty() {
                show_error("Please fill Session ID, Exercise Code, and Set Numbers.");
                return;
            }

            let set_numbers = match parse_set_numbers(&setnums_raw) {
                Ok(v) if !v.is_empty() => v,
                _ => {
                    show_error("Invalid Set Numbers. Use comma-separated integers and ranges, e.g., 1,2,4-6.");
                    return;
                }
            };

            // Build set keys
            let set_keys: Vec<SetKey> = set_numbers.into_iter().map(|n| SetKey { session_id: session_id.clone(), ex_code: ex_code.clone(), set_num: n }).collect();

            // Collect media files (store just the file names)
            let mut media_files: Vec<String> = vec![];
            let files = file_chooser.files();
            for i in 0..files.n_items() {
                if let Some(obj) = files.item(i) {
                    if let Ok(gfile) = obj.downcast::<gio::File>() {
                        if let Some(fname) = gfile.basename() {
                            media_files.push(fname.to_string_lossy().to_string());
                        }
                    }
                }
            }
            if media_files.is_empty() {
                show_error("Please select one or more media files.");
                return;
            }

            let writer = MediaAttachmentWriter::new(paths.media_attachments_path());
            let result = if do_detach {
                writer.detach_many(&set_keys, &media_files)
            } else {
                writer.attach_many(&set_keys, &media_files)
            };

            match result {
                Ok(()) => {},
                Err(e) => {
                    show_error(&format!("Failed to write media attachments: {}", e));
                    return;
                }
            }
        }
        dialog.close();
    });

    dialog.present();
}

fn parse_set_numbers(input: &str) -> Result<Vec<u32>, ()> {
    let mut out = Vec::new();
    for part in input.split(',') {
        let p = part.trim();
        if p.is_empty() { continue; }
        if let Some((a, b)) = p.split_once('-') {
            let start: u32 = a.trim().parse().map_err(|_| ())?;
            let end: u32 = b.trim().parse().map_err(|_| ())?;
            if start > end { return Err(()); }
            for n in start..=end { out.push(n); }
        } else {
            let n: u32 = p.parse().map_err(|_| ())?;
            out.push(n);
        }
    }
    Ok(out)
}

fn show_error(message: &str) {
    use gtk4::{Dialog, DialogFlags, ResponseType};
    let d = Dialog::with_buttons(
        Some("Error"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("OK", ResponseType::Ok)]
    );
    crate::ui::util::standardize_dialog(&d);
    let lbl = Label::builder().label(message).wrap(true).build();
    d.content_area().append(&lbl);
    d.connect_response(|d, _| d.close());
    d.present();
}
