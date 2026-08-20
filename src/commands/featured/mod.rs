mod args;
pub use args::*;

use anyhow::Result;
use colored::Colorize;

use super::require_client;

pub async fn run(args: FeaturedArgs) -> Result<()> {
    match args.command {
        FeaturedCommand::List { json } => list(json).await,
        FeaturedCommand::AddSpeaker { id } => add_speaker(&id).await,
        FeaturedCommand::RemoveSpeaker { id, yes } => remove_speaker(&id, yes).await,
        FeaturedCommand::AddTalk { id } => add_talk(&id).await,
        FeaturedCommand::RemoveTalk { id, yes } => remove_talk(&id, yes).await,
    }
}

async fn list(json: bool) -> Result<()> {
    let client = require_client()?;
    let speakers: serde_json::Value = client.query("featured.admin.listSpeakers", None).await?;
    let talks: serde_json::Value = client.query("featured.admin.listTalks", None).await?;

    if json {
        let out = serde_json::json!({
            "speakers": speakers,
            "talks": talks,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("{}", "── Featured Speakers ──".bold().cyan());
        if let Some(arr) = speakers.as_array() {
            if arr.is_empty() {
                println!("No featured speakers.");
            }
            for s in arr {
                println!(
                    "- {}",
                    s.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown")
                );
            }
        }

        println!("\n{}", "── Featured Talks ──".bold().cyan());
        if let Some(arr) = talks.as_array() {
            if arr.is_empty() {
                println!("No featured talks.");
            }
            for t in arr {
                println!(
                    "- {}",
                    t.get("title").and_then(|n| n.as_str()).unwrap_or("Unknown")
                );
            }
        }
    }

    Ok(())
}

async fn add_speaker(id: &str) -> Result<()> {
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "featured.admin.addSpeaker",
            &serde_json::json!({ "speakerId": id }),
        )
        .await?;
    println!("Successfully added speaker {id} to featured list.");
    Ok(())
}

async fn remove_speaker(id: &str, yes: bool) -> Result<()> {
    if !yes {
        if !console::Term::stdout().is_term() {
            anyhow::bail!("Confirmation required in non-interactive mode. Pass -y to confirm.");
        }
        let confirmed = dialoguer::Confirm::new()
            .with_prompt(format!("Remove speaker {id} from front page?"))
            .default(false)
            .interact()?;

        if !confirmed {
            anyhow::bail!("Removal cancelled.");
        }
    }

    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "featured.admin.removeSpeaker",
            &serde_json::json!({ "speakerId": id }),
        )
        .await?;
    println!("Successfully removed speaker {id} from featured list.");
    Ok(())
}

async fn add_talk(id: &str) -> Result<()> {
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "featured.admin.addTalk",
            &serde_json::json!({ "talkId": id }),
        )
        .await?;
    println!("Successfully added talk {id} to featured list.");
    Ok(())
}

async fn remove_talk(id: &str, yes: bool) -> Result<()> {
    if !yes {
        if !console::Term::stdout().is_term() {
            anyhow::bail!("Confirmation required in non-interactive mode. Pass -y to confirm.");
        }
        let confirmed = dialoguer::Confirm::new()
            .with_prompt(format!("Remove talk {id} from front page?"))
            .default(false)
            .interact()?;

        if !confirmed {
            anyhow::bail!("Removal cancelled.");
        }
    }

    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "featured.admin.removeTalk",
            &serde_json::json!({ "talkId": id }),
        )
        .await?;
    println!("Successfully removed talk {id} from featured list.");
    Ok(())
}
