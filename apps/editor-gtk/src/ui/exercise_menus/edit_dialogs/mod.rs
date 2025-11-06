// Edit dialogs module organization

pub mod dialog_components;
pub mod superset_edit;
pub mod circuit_edit; 
pub mod scheme_edit;
pub mod complex_edit;

// Re-export public functions for external use
pub use superset_edit::show_edit_superset_dialog;
pub use circuit_edit::show_edit_circuit_dialog;
pub use scheme_edit::show_edit_scheme_dialog;
pub use complex_edit::show_edit_complex_dialog;