
fn create_inspector() -> Box {
    let inspector_box = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .spacing(8)
        .build();

    // Inspector header
    let inspector_label = Label::builder()
        .label("Inspector")
        .css_classes(vec!["heading".to_string()])
        .halign(gtk4::Align::Start)
        .build();

    inspector_box.append(&inspector_label);

    // Inspector content (scrolled)
    let scrolled = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .has_frame(true)
        .build();

    let placeholder = Label::builder()
        .label("Select an item to edit")
        .css_classes(vec!["dim-label".to_string()])
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .build();

    scrolled.set_child(Some(&placeholder));
    inspector_box.append(&scrolled);

    inspector_box
}
