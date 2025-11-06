use crate::state::AppState;
use crate::operations::segment::add_custom_comment_to_plan;
use gtk4::{Dialog, DialogFlags, ResponseType, Box as GtkBox, Orientation, Label, TextView, ScrolledWindow, DropDown, StringList};
use gtk4::prelude::*;
use glib::clone;
use std::sync::{Arc, Mutex};

pub fn show_add_comment_dialog(state: Arc<Mutex<AppState>>) {
    let dialog = Dialog::with_buttons(
        Some("Add Comment"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Add", ResponseType::Accept)]
    );
    
    dialog.set_default_size(400, 300);
    crate::ui::util::standardize_dialog(&dialog);
    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .spacing(12)
        .build();
    
    // Icon dropdown
    let icon_label = Label::new(Some("Icon (optional):"));
    let exercise_icons = StringList::new(&[
        "🏋️", "💪", "🔥", "⚡", "🎯", "📝", "⏱️", "🕐", "⏰", "⏲️", "💥", "🚀", "⭐", "🔔", "⚠️", 
        "✅", "🔴", "🟡", "🟢", "🔵", "⚫", "📊", "📈", "📉", "🎖️", "🏆", "💯", "🔄", "⏳"
    ]);
    
    let icon_dropdown = DropDown::new(Some(exercise_icons), None::<gtk4::Expression>);
    icon_dropdown.set_selected(0); // Default to "🏋️"
    
    content.append(&icon_label);
    content.append(&icon_dropdown);
    
    // Comment text
    let text_label = Label::new(Some("Comment Text:"));
    content.append(&text_label);
    
    let text_view = TextView::builder()
        .wrap_mode(gtk4::WrapMode::Word)
        .build();
    
    let text_buffer = text_view.buffer();
    text_buffer.set_text("Rest between exercises - Death to Windows!");
    
    let scrolled = ScrolledWindow::builder()
        .child(&text_view)
        .min_content_height(100)
        .build();
    
    content.append(&scrolled);
    
    dialog.content_area().append(&content);
    // Ctrl+Enter triggers Add
    crate::ui::util::bind_ctrl_enter_to_accept(&dialog);
    
    dialog.connect_response(clone!(@strong state, @strong icon_dropdown => move |dialog, response| {
        if response == ResponseType::Accept {
            let selected_index = icon_dropdown.selected();
            let icon_options = ["🏋️", "💪", "🔥", "⚡", "🎯", "📝", "⏱️", "🕐", "⏰", "⏲️", "💥", "🚀", "⭐", "🔔", "⚠️", 
                               "✅", "🔴", "🟡", "🟢", "🔵", "⚫", "📊", "📈", "📉", "🎖️", "🏆", "💯", "🔄", "⏳"];
            let selected_icon = icon_options[selected_index as usize];
            let icon = Some(selected_icon.to_string());
            
            let text_buffer = text_view.buffer();
            let (start, end) = text_buffer.bounds();
            let text = text_buffer.text(&start, &end, false).to_string();
            
            add_custom_comment_to_plan(state.clone(), text, icon);
        }
        dialog.close();
    }));
    
    dialog.present();
}
