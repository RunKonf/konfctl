pub mod auth;
pub mod client;
pub mod commands;
pub mod config;
pub mod display;
pub mod template;
pub mod types;
pub mod ui;

static IS_AGENT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

pub fn set_is_agent(agent: bool) {
    let _ = IS_AGENT.set(agent);
}

pub fn is_agent() -> bool {
    *IS_AGENT.get().unwrap_or(&false)
}
