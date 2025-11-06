use gtk4::RecentManager;
use libadwaita::gio::{Menu, MenuItem};
use gtk4::prelude::{RecentManagerExt, ToVariant, FileExt};
use crate::services::recent_files::RecentFilesService;

#[allow(dead_code)]
pub fn create_recent_menu() -> Menu {
    let recent_menu = Menu::new();
    populate_recent_menu(&recent_menu, 5, &["application/json"]);
    
    // Set up automatic updates
    let recent_mgr = RecentManager::default();
    let menu_clone = recent_menu.clone();
    recent_mgr.connect_changed(move |_| {
        populate_recent_menu(&menu_clone, 5, &["application/json"]);
    });
    
    recent_menu
}

#[allow(dead_code)]
pub fn populate_recent_menu(menu: &Menu, max_items: usize, mime_whitelist: &[&str]) {
    populate_recent_menu_with_service(menu, max_items, mime_whitelist, None);
}

pub fn populate_recent_menu_with_service(menu: &Menu, max_items: usize, mime_whitelist: &[&str], service: Option<&RecentFilesService>) {
    menu.remove_all();

    // Gather known infos once and map by URI for quick lookup
    let infos = RecentManager::default().items();
    let mut by_uri: std::collections::HashMap<String, gtk4::RecentInfo> = std::collections::HashMap::new();
    for info in infos.into_iter() {
        by_uri.insert(info.uri().to_string(), info);
    }

    // Determine ordered URIs: prefer service MRU if provided and non-empty
    let ordered_uris: Vec<String> = if let Some(svc) = service {
        let uris = svc.get_all_uris();
        if !uris.is_empty() { uris } else { by_uri.keys().cloned().collect() }
    } else {
        by_uri.keys().cloned().collect()
    };

    // If falling back to raw recent manager keys, sort by visited desc for stability
    let mut ordered_uris = ordered_uris;
    if service.is_none() || ordered_uris.len() == by_uri.len() {
        ordered_uris.sort_by(|ua, ub| {
            let ia = by_uri.get(ua);
            let ib = by_uri.get(ub);
            match (ia, ib) {
                (Some(a), Some(b)) => {
                    let va = a.visited().to_unix();
                    let vb = b.visited().to_unix();
                    vb.cmp(&va)
                        .then_with(|| {
                            let ma = a.modified().to_unix();
                            let mb = b.modified().to_unix();
                            mb.cmp(&ma)
                        })
                        .then_with(|| {
                            let aa = a.added().to_unix();
                            let ab = b.added().to_unix();
                            ab.cmp(&aa)
                        })
                }
                _ => std::cmp::Ordering::Equal,
            }
        });
    }

    let mut added = 0usize;
    for uri in ordered_uris.into_iter() {
        let info_opt = by_uri.get(&uri);

        // MIME filtering and existence checks
        if let Some(info) = info_opt {
            if !info.exists() || !info.is_local() { continue; }
            if !mime_whitelist.is_empty() && !mime_whitelist.contains(&info.mime_type().as_str()) { continue; }
        } else {
            // If we don't have info, do a light filter by extension when whitelist is set
            if !mime_whitelist.is_empty() && !uri.to_lowercase().ends_with(".json") { continue; }
        }

        // Label from info if available, else fallback to filename from URI
        let label = if let Some(info) = info_opt {
            info.short_name().to_string()
        } else {
            // Fallback: try to parse a name from URI
            if let Some(gf) = libadwaita::gio::File::for_uri(&uri).path() {
                gf.file_name()
                    .and_then(|os| os.to_str())
                    .unwrap_or(&uri)
                    .to_string()
            } else {
                uri.clone()
            }
        };

        let display_label = if added < 9 {
            format!("{}. {}", added + 1, label)
        } else {
            label
        };

        let item = MenuItem::new(Some(&display_label), None);
        if added < 9 {
            let action_name = format!("win.open_recent_{}", added + 1);
            item.set_action_and_target_value(Some(&action_name), None);
        } else {
            item.set_action_and_target_value(Some("win.open_recent"), Some(&uri.to_variant()));
        }
        menu.append_item(&item);

        added += 1;
        if added >= max_items { break; }
    }

    // Do not mutate service MRU here; ordering comes from service

    if added == 0 {
        menu.append(Some("(No recent files)"), None);
    }
}
