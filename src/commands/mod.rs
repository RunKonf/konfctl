use anyhow::{Context, Result};

use crate::client::TrpcClient;
use crate::config;

pub mod admin_status;
pub mod agent_discovery;
pub mod agents;
pub mod featured;
pub mod login;
pub mod logout;
pub mod messages;
pub mod proposals;
pub mod schedule;
pub mod speakers;
pub mod sponsors;
pub mod status;

pub fn require_client() -> Result<TrpcClient> {
    let cfg = config::load().context("Not logged in. Run `konf login` first.")?;
    Ok(TrpcClient::from_config(&cfg))
}
