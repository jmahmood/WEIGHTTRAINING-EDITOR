use glib::clone;
use gtk4::gdk;
use gtk4::prelude::IsA;
use gtk4::ResponseType;
use gtk4::{gdk::Key, gdk::ModifierType, Button, EventControllerKey, PropagationPhase};
use gtk4::{prelude::*, Application, ApplicationWindow, Dialog};
use gtk4::{Box as GtkBox, Image, Label, Orientation, Popover};

/// Returns the currently active `ApplicationWindow`, if any.
pub fn parent_for_dialog() -> Option<ApplicationWindow> {
    let app = Application::default();
    app.active_window()
        .and_then(|w| w.downcast::<ApplicationWindow>().ok())
}

/// Apply standard flags and transiency to any dialog-like widget.
/// - Sets transient-for to the active window (if available)
/// - Sets modal = true
/// - Sets destroy-with-parent = true
pub fn standardize_dialog<D: IsA<Dialog>>(dialog: &D) {
    let dlg: &Dialog = dialog.as_ref();
    if let Some(parent) = parent_for_dialog() {
        dlg.set_transient_for(Some(&parent));
        dlg.set_destroy_with_parent(true);
    }
    dlg.set_modal(true);
}

/// Bind Ctrl+Enter to trigger the given button's click from anywhere under `root`.
pub fn bind_ctrl_enter_to_button<W: IsA<gtk4::Widget>>(root: &W, button: &Button) {
    let key = EventControllerKey::new();
    key.set_propagation_phase(PropagationPhase::Capture);
    let btn = button.clone();
    key.connect_key_pressed(clone!(@strong btn => @default-return glib::Propagation::Proceed, move |_, keyval, _, state| {
        if (keyval == Key::Return || keyval == Key::KP_Enter) && state.contains(ModifierType::CONTROL_MASK) {
            btn.emit_by_name::<()>("clicked", &[]);
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    }));
    root.as_ref().add_controller(key);
}

/// Convenience: find the primary response button (Accept/Ok/Yes) and bind Ctrl+Enter.
/// Returns true if a button was found and bound.
pub fn bind_ctrl_enter_to_accept<D: IsA<Dialog>>(dialog: &D) -> bool {
    let dlg: &Dialog = dialog.as_ref();
    let candidates = [ResponseType::Accept, ResponseType::Ok, ResponseType::Yes];
    for rt in candidates {
        if let Some(w) = dlg.widget_for_response(rt) {
            if let Ok(btn) = w.downcast::<Button>() {
                bind_ctrl_enter_to_button(dlg, &btn);
                return true;
            }
        }
    }
    false
}

/// Show a non-blocking warning dialog informing about unsaved changes.
#[allow(dead_code)]
pub fn show_discard_changes_warning() {
    use gtk4::{ButtonsType, DialogFlags, MessageDialog, MessageType};
    let dlg = MessageDialog::new(
        parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        MessageType::Warning,
        ButtonsType::Ok,
        "You have unsaved changes. Press Ctrl+Enter to save, or Close to discard.",
    );
    standardize_dialog(&dlg);
    dlg.connect_response(|d, _| d.close());
    dlg.present();
}

/// Ask user whether to discard unsaved changes; returns true if discard confirmed.
#[allow(dead_code)]
pub fn confirm_discard_changes() -> bool {
    use gtk4::{ButtonsType, DialogFlags, MessageDialog, MessageType, ResponseType};
    let dlg = MessageDialog::new(
        parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        MessageType::Question,
        ButtonsType::YesNo,
        "Discard unsaved changes?",
    );
    standardize_dialog(&dlg);
    // Use a nested main loop to block until user responds
    let decision = std::rc::Rc::new(std::cell::Cell::new(false));
    let done = std::rc::Rc::new(std::cell::Cell::new(false));
    let decision_clone = decision.clone();
    let done_clone = done.clone();
    dlg.connect_response(move |d, resp| {
        decision_clone.set(resp == ResponseType::Yes);
        done_clone.set(true);
        d.close();
    });
    dlg.present();
    let ctx = glib::MainContext::default();
    while !done.get() {
        ctx.iteration(true);
    }
    decision.get()
}

/// Show a simple right-click context menu with a single "Edit" option.
/// - Anchors to `relative_to` and appears near the click position (x,y).
/// - Invokes `on_edit` when the user activates the menu item.
pub fn show_edit_context_menu<W: IsA<gtk4::Widget> + Clone + 'static>(
    relative_to: &W,
    x: f64,
    y: f64,
    on_edit: Box<dyn Fn() + 'static>,
) {
    // Build a minimal popover menu
    let popover = Popover::new();
    popover.set_autohide(true);
    popover.set_has_arrow(true);
    // Attach to the given widget so the popover positions relative to it
    popover.set_parent(relative_to);

    // Position near cursor
    let rect = gdk::Rectangle::new(x as i32, y as i32, 1, 1);
    popover.set_pointing_to(Some(&rect));

    // Content: simple row behaving like a menu item
    let box_ = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();

    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .margin_start(12)
        .margin_end(12)
        .margin_top(8)
        .margin_bottom(8)
        .build();
    let icon = Image::from_icon_name("document-edit-symbolic");
    icon.set_icon_size(gtk4::IconSize::Normal);
    let label = Label::new(Some("Edit"));
    label.set_halign(gtk4::Align::Start);
    row.append(&icon);
    row.append(&label);
    box_.append(&row);

    // Activate on click: close the popover, then run callback
    let click = gtk4::GestureClick::new();
    click.set_button(1);
    let popover_for_click = popover.clone();
    click.connect_pressed(move |g, _n, _x, _y| {
        // Stop propagation so it doesn't re-trigger other handlers
        g.set_state(gtk4::EventSequenceState::Claimed);
        popover_for_click.popdown();
        on_edit();
    });
    box_.add_controller(click);

    popover.set_child(Some(&box_));
    popover.popup();
}
