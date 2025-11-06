// Exercise menu dialogs module
// Provides dialogs for adding and editing various exercise types

pub mod add_dialogs;
pub mod complex_dialogs;
pub mod complex;
pub mod edit_dialogs;
pub mod exercise_data;

// Re-export all public functions for backward compatibility
pub use add_dialogs::{
    show_add_rpe_set_dialog,
    show_add_percentage_set_dialog,
    show_add_amrap_dialog,
    show_add_time_based_dialog,
};

// New structured exports for dialogs split by type
pub use complex::superset::show_add_superset_dialog;
pub use complex::circuit::show_add_circuit_dialog;
// Keep remaining complex dialogs via legacy module for now
pub use complex::scheme::show_add_scheme_dialog;
pub use complex::complex::show_add_complex_dialog;
pub use complex_dialogs::{
    show_add_group_choose_dialog,
    show_add_group_rotate_dialog,
    show_add_group_optional_dialog,
    show_add_group_superset_dialog,
};

pub use edit_dialogs::{
    show_edit_superset_dialog,
    show_edit_circuit_dialog,
    show_edit_scheme_dialog,
    show_edit_complex_dialog,
};

// Data structures are used internally by the dialogs
// If needed externally, uncomment the lines below:
// pub use exercise_data::{
//     SupersetExerciseData,
//     CircuitExerciseData,
// };
