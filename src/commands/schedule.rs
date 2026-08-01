use anyhow::Result;
use clap::{Args, Subcommand};
use colored::Colorize;

use super::require_client;
use crate::types::{Schedule, ScheduleStatus};

#[derive(Subcommand)]
pub enum ScheduleCommand {
    /// List all schedules (draft, official, archived)
    List(ListArgs),
    /// Get a specific schedule by ID
    Get {
        /// Schedule ID
        id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Promote a draft schedule to official
    Promote {
        /// Schedule ID
        id: String,
    },
    /// Save or update a schedule via JSON payload
    Save {
        /// The JSON payload containing the schedule to save
        payload: String,
    },
    /// Delete a schedule
    Delete {
        /// Schedule ID
        id: String,
        /// Skip confirmation
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

#[derive(Args, Clone)]
pub struct ListArgs {
    /// Filter by status
    #[arg(long, value_enum)]
    pub status: Option<ScheduleStatus>,
}

pub async fn list(args: ListArgs) -> Result<()> {
    let client = require_client()?;
    let mut payload = serde_json::Map::new();
    if let Some(status) = args.status {
        payload.insert("status".to_string(), serde_json::json!(status));
    }
    let payload = serde_json::Value::Object(payload);
    
    let schedules: Vec<Schedule> = client
        .query("schedule.admin.list", Some(&payload))
        .await?;

    if crate::is_agent() {
        println!("{}", serde_json::to_string(&schedules)?);
        return Ok(());
    }

    let unhandled = crate::display::print_json_list(
        schedules,
        None,
        false,
        false,
        |s| {
            serde_json::json!({
                "id": s.id,
                "date": s.date,
                "status": s.status.clone().map(|st| st.to_string()).unwrap_or_else(|| "unknown".to_string()),
                "version": s.version.unwrap_or(0),
                "tracks": s.tracks.as_ref().map(|t| t.len()).unwrap_or(0)
            })
        },
    )?;

    if let Some(items) = unhandled {
        println!("{:<35} | {:<12} | {:<10} | {}", "ID", "DATE", "STATUS", "VERSION");
        println!("{}", "-".repeat(70));
        for s in items {
            let status = s.status.map(|st| st.to_string()).unwrap_or_else(|| "unknown".to_string());
            let version = s.version.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string());
            println!("{:<35} | {:<12} | {:<10} | {}", s.id, s.date, status, version);
        }
    }
    Ok(())
}

pub async fn get(id: &str, json: bool) -> Result<()> {
    let client = require_client()?;
    let payload = serde_json::json!({ "id": id });
    let schedule: Schedule = client
        .query("schedule.admin.getById", Some(&payload))
        .await?;

    if json || crate::is_agent() {
        println!("{}", serde_json::to_string_pretty(&schedule)?);
    } else {
        println!("ID: {}", schedule.id);
        println!("Date: {}", schedule.date);
        println!("Status: {}", schedule.status.map(|s| s.to_string()).unwrap_or_else(|| "unknown".to_string()));
        println!("Version: {}", schedule.version.unwrap_or(0));
        let track_count = schedule.tracks.map(|t| t.len()).unwrap_or(0);
        println!("Tracks: {}", track_count);
    }
    Ok(())
}

pub async fn promote(id: &str) -> Result<()> {
    let client = require_client()?;
    let payload = serde_json::json!({
        "id": id,
        "action": "promote"
    });
    
    let res: serde_json::Value = client.mutate("schedule.action", &payload).await?;

    if crate::is_agent() {
        println!("{}", serde_json::to_string(&res)?);
    } else {
        println!("Successfully promoted schedule {} to Official.", id);
    }
    Ok(())
}

pub async fn delete(id: &str, yes: bool) -> Result<()> {
    if !yes && console::Term::stdout().is_term() {
        let confirmed = dialoguer::Confirm::new()
            .with_prompt(format!("Are you sure you want to delete schedule {}?", id))
            .default(false)
            .interact()?;

        if !confirmed {
            anyhow::bail!("Deletion cancelled.");
        }
    }

    let client = require_client()?;
    let payload = serde_json::json!({ "id": id });
    let _res: serde_json::Value = client.mutate("schedule.admin.delete", &payload).await?;

    if crate::is_agent() {
        println!("{}", serde_json::json!({ "ok": true, "id": id }));
    } else {
        println!("Successfully deleted schedule {}.", id);
    }
    Ok(())
}

pub async fn save(payload: &str) -> Result<()> {
    let client = require_client()?;
    let payload: serde_json::Value = serde_json::from_str(payload).map_err(|e| anyhow::anyhow!("Invalid JSON payload: {}", e))?;
    
    let res: serde_json::Value = client.mutate("schedule.save", &payload).await?;

    if crate::is_agent() {
        println!("{}", serde_json::to_string(&res)?);
    } else {
        println!("Successfully saved schedule.");
    }
    Ok(())
}
