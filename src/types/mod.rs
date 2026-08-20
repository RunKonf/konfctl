mod agent_config;
pub mod email_template;
pub mod message;
pub mod proposal;
pub mod schedule;
pub mod speaker;
pub mod sponsor;
pub mod status;

pub use agent_config::*;
pub use email_template::*;
pub use message::*;
pub use proposal::*;
pub use schedule::ScheduleStatus;
pub use schedule::*;
pub use speaker::*;
pub use sponsor::*;
pub use status::*;

use serde::Deserialize;

/// Deserialize a `Vec<T>` that may be `null` in JSON (common with Sanity CMS).
/// Maps both missing fields and explicit `null` to an empty `Vec`.
pub fn null_to_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<Vec<T>>::deserialize(deserializer).map(Option::unwrap_or_default)
}
