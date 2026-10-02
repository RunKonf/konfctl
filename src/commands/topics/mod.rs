use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;

use crate::commands::require_client;

#[derive(Subcommand)]
pub enum TopicCommand {
    /// List all conference topics
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Topic {
    pub _id: String,
    pub title: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

pub async fn list(json: bool) -> Result<()> {
    let client = require_client()?;
    let sp = crate::ui::spinner("Fetching topics…");
    let topics: Vec<Topic> = client.query("topic.list", None).await?;
    sp.finish_and_clear();

    if json || crate::is_agent() {
        if crate::is_agent() {
            println!("{}", serde_json::to_string(&topics)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&topics)?);
        }
    } else {
        if topics.is_empty() {
            println!("No topics found.");
            return Ok(());
        }

        println!("{:<35} {:<30}", "ID".bold().cyan(), "TITLE".bold().cyan());
        for t in &topics {
            println!("{:<35} {:<30}", t._id, t.title);
        }
    }
    Ok(())
}
