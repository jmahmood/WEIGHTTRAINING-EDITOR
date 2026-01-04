pub mod attachments;
pub mod charts;
pub mod export;
pub mod location;
pub mod models;
pub mod paths;
pub mod schemes;
pub mod time;
pub mod versioning;

#[cfg(test)]
mod location_test;

pub use attachments::*;
pub use charts::*;
pub use export::*;
pub use models::*;
pub use paths::*;
pub use schemes::*;
pub use time::*;
pub use versioning::*;

/// The JSON schema for plan validation
pub const PLAN_SCHEMA_V0_3: &str = include_str!("../schema/plan_v0_3.json");
pub const PLAN_SCHEMA_V0_4: &str = include_str!("../schema/plan_v0_4.json");
