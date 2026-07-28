mod args;
pub use args::*;

use anyhow::Result;
use colored::Colorize;

use super::require_client;
use crate::types::AgentConfig;

pub async fn run(args: AgentArgs) -> Result<()> {
    match args.command {
        AgentCommand::Get { json } => get(json).await,
        AgentCommand::Set(set_args) => set(set_args).await,
    }
}

pub async fn get(json: bool) -> Result<()> {
    let client = require_client()?;
    let config: AgentConfig = client.query("agents.get", None).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&config)?);
        return Ok(());
    }

    println!("{}", "── Agent Configuration ──".bold().cyan());
    println!(
        "{}",
        "Hint: Use this context to guide your task execution and tone.".dimmed()
    );

    if let Some(ctx) = config.conference_context {
        println!("\n{}:", "Conference Context".bold());
        println!("{ctx}");
    }

    if let Some(rev) = config.proposal_review_config {
        println!("\n{}:", "Proposal Review Config".bold());
        println!("{rev}");
    }

    if let Some(crm) = config.sponsor_crm_config {
        println!("\n{}:", "Sponsor CRM Config".bold());
        println!("{crm}");
    }

    println!("\n{}", "── Agent Commands ──".dimmed());
    println!(
        "{}",
        "• Update context: konf agents set --context \"...\"".dimmed()
    );
    println!(
        "{}",
        "• Update rules:   konf agents set --review-config \"...\"".dimmed()
    );

    Ok(())
}

pub async fn set(args: SetArgs) -> Result<()> {
    let client = require_client()?;

    let input = serde_json::json!({
        "conferenceContext": args.context,
        "proposalReviewConfig": args.review_config,
        "sponsorCrmConfig": args.crm_config,
    });

    client
        .mutate::<serde_json::Value>("agents.update", &input)
        .await?;

    println!("Agent configuration updated successfully.");
    Ok(())
}
