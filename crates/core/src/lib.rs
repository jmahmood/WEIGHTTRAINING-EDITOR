pub mod models;
pub mod paths;
pub mod schemes;
pub mod location;
pub mod versioning;
pub mod export;
pub mod charts;
pub mod time;
pub mod attachments;

#[cfg(test)]
mod location_test;

pub use models::*;
pub use paths::*;
pub use schemes::*;
pub use versioning::*;
pub use export::*;
pub use charts::*;
pub use time::*;
pub use attachments::*;

/// The JSON schema for plan validation
pub const PLAN_SCHEMA_V0_3: &str = include_str!("../schema/plan_v0_3.json");
