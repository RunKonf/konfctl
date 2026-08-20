mod args;
mod interactive;
pub use args::*;

use anyhow::Result;
use colored::Colorize;

use super::require_client;
use crate::client::TrpcClient;
use crate::types::{Proposal, ProposalStatus, Speaker, SpeakerSummary};
use crate::ui;

pub async fn fetch_all(client: &TrpcClient) -> Result<Vec<SpeakerSummary>> {
    client.query("speaker.admin.list", None).await
}

pub async fn fetch_search(client: &TrpcClient, args: &ListArgs) -> Result<Vec<Speaker>> {
    client
        .query("speaker.admin.search", Some(&serde_json::to_value(args)?))
        .await
}

pub async fn fetch_one(client: &TrpcClient, id: &str) -> Result<Speaker> {
    client
        .query(
            "speaker.admin.getById",
            Some(&serde_json::json!({ "id": id })),
        )
        .await
}

pub async fn fetch_conference_speakers(
    client: &TrpcClient,
    args: &ListArgs,
) -> Result<Vec<SpeakerSummary>> {
    let statuses = args
        .status
        .clone()
        .unwrap_or_else(|| vec![ProposalStatus::Accepted, ProposalStatus::Confirmed]);

    let proposal_args = crate::commands::proposals::ListArgs {
        status: Some(statuses),
        search: args.query.clone(),
        sort_by: args.sort,
        sort_order: args.order,
        ..Default::default()
    };

    let proposals = crate::commands::proposals::fetch_all(client, &proposal_args).await?;

    let mut speakers: Vec<SpeakerSummary> = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for p in proposals {
        for s in p.speakers {
            if seen_ids.insert(s.id.clone()) {
                speakers.push(SpeakerSummary {
                    id: s.id,
                    name: s.name,
                    email: s.email.as_str().map(String::from),
                    title: s.title.as_str().map(String::from),
                    slug: s.slug.as_str().map(String::from),
                    image: s.image.as_str().map(String::from),
                });
            }
        }
    }
    Ok(speakers)
}

pub async fn fetch_talks_for_speaker(client: &TrpcClient, id: &str) -> Result<Vec<Proposal>> {
    let proposal_args = crate::commands::proposals::ListArgs {
        status: Some(vec![
            ProposalStatus::Submitted,
            ProposalStatus::Accepted,
            ProposalStatus::Confirmed,
            ProposalStatus::Waitlisted,
            ProposalStatus::Rejected,
        ]),
        ..Default::default()
    };
    let all_proposals = crate::commands::proposals::fetch_all(client, &proposal_args).await?;
    let speaker_talks: Vec<Proposal> = all_proposals
        .into_iter()
        .filter(|p| p.speakers.iter().any(|s| s.id == id))
        .collect();
    Ok(speaker_talks)
}

#[allow(clippy::too_many_lines)]
pub async fn list(args: ListArgs) -> Result<()> {
    let client = require_client()?;

    // Global list/search across all speakers
    if args.all {
        let all = fetch_all(&client).await?;
        let filtered = if let Some(ref q) = args.query {
            let q = q.to_lowercase();
            all.into_iter()
                .filter(|s| {
                    s.name.to_lowercase().contains(&q)
                        || s.email
                            .as_ref()
                            .is_some_and(|e| e.to_lowercase().contains(&q))
                })
                .collect()
        } else {
            all
        };

        let unhandled =
            crate::display::print_json_list(filtered, args.limit, args.compact, args.json, |s| {
                serde_json::json!({
                    "id": s.id,
                    "name": s.name,
                    "email": s.email,
                    "title": s.title,
                })
            })?;

        if let Some(filtered) = unhandled {
            if console::Term::stdout().is_term() {
                return interactive::list_interactive(&client, &filtered).await;
            }
            if filtered.is_empty() {
                println!("No speakers found matching the criteria.");
                return Ok(());
            }
            println!(
                "{}",
                "ID                   NAME                 EMAIL"
                    .bold()
                    .cyan()
            );
            for s in filtered {
                println!(
                    "{:<20} {:<20} {}",
                    s.id,
                    s.name,
                    s.email.as_deref().unwrap_or_default()
                );
            }
        }
        return Ok(());
    }

    // Fetch conference speakers for interactive mode or if no filters are applied
    if !args.json && !args.has_cli_filters() && console::Term::stdout().is_term() {
        let speakers = fetch_conference_speakers(&client, &args).await?;
        return interactive::list_interactive(&client, &speakers).await;
    }

    let speakers = fetch_conference_speakers(&client, &args).await?;

    let unhandled =
        crate::display::print_json_list(speakers, args.limit, args.compact, args.json, |s| {
            serde_json::json!({
                "id": s.id,
                "name": s.name,
                "email": s.email,
                "title": s.title,
            })
        })?;

    if let Some(speakers) = unhandled {
        if speakers.is_empty() {
            println!("No speakers found with the given filters.");
            return Ok(());
        }

        println!(
            "{}",
            "ID                   NAME                 EMAIL"
                .bold()
                .cyan()
        );
        for s in speakers {
            println!(
                "{:<20} {:<20} {}",
                s.id,
                s.name,
                s.email.as_deref().unwrap_or_default()
            );
        }
        println!(
            "\n{}",
            "Hint: Use `konf admin speakers get <ID>` for full bio and links.".dimmed()
        );
    }
    Ok(())
}

pub async fn get(id: &str, json: bool) -> Result<()> {
    let client = require_client()?;
    let speaker = fetch_one(&client, id).await?;
    let speaker_talks = fetch_talks_for_speaker(&client, id).await?;

    if json || crate::is_agent() {
        let mut out = serde_json::to_value(&speaker)?;
        out["talks"] = serde_json::to_value(speaker_talks)?;
        if crate::is_agent() {
            println!("{}", serde_json::to_string(&out)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
    } else {
        println!("{} ({})", speaker.name.bold(), speaker.id.dimmed());
        if let Some(email) = speaker.email.as_str() {
            println!("Email:   {email}");
        }
        if let Some(company) = speaker.company.as_str() {
            println!("Company: {company}");
        }
        if let Some(title) = speaker.title.as_str() {
            println!("Title:   {title}");
        }

        if !speaker.flags.is_empty() {
            println!("Flags:   {:?}", speaker.flags);
        }

        if !speaker.links.is_empty() {
            println!("\nLinks:");
            for link in &speaker.links {
                println!("  - {link}");
            }
        }

        if !speaker_talks.is_empty() {
            println!("\nTalks:");
            for talk in speaker_talks {
                let status = format!("[{}]", talk.status);
                println!("  - {} {}", status.yellow(), talk.title);
            }
        }

        if !speaker.bio.is_null() {
            println!("\nBio:");
            if speaker.bio.is_array() {
                println!(
                    "{}",
                    crate::types::portable_text_to_plain(speaker.bio.as_array().unwrap())
                );
            } else if speaker.bio.is_string() {
                println!("{}", speaker.bio.as_str().unwrap());
            } else {
                println!("{}", speaker.bio);
            }
        }
    }
    Ok(())
}

pub async fn add(args: CreateArgs) -> Result<()> {
    let client = require_client()?;

    // Interactive mode if no name is provided
    if args.name.is_empty() && console::Term::stdout().is_term() {
        return interactive::add_wizard(&client).await;
    }

    let mut payload = serde_json::to_value(&args)?;

    // Wrap plain text description in a Portable Text array if provided
    if let Some(ref bio) = args.bio {
        let portable_text = serde_json::json!([{
            "_type": "block",
            "children": [{
                "_type": "span",
                "text": bio
            }],
            "style": "normal"
        }]);
        payload["bio"] = portable_text;
    }

    let speaker: Speaker = client.mutate("speaker.admin.create", &payload).await?;
    if crate::is_agent() {
        println!("{}", serde_json::json!({ "ok": true, "id": speaker.id }));
    } else {
        println!(
            "Successfully created speaker {} (ID: {})",
            speaker.name, speaker.id
        );
    }
    Ok(())
}

pub async fn find_or_create(args: FindOrCreateArgs) -> Result<()> {
    let client = require_client()?;

    // 1. Try to find by email
    let sp = ui::spinner("Checking if speaker exists…");
    let search_args = ListArgs {
        query: Some(args.email.clone()),
        all: true,
        ..Default::default()
    };
    let results = fetch_search(&client, &search_args).await?;
    sp.finish_and_clear();

    if let Some(speaker) = results.first() {
        if crate::is_agent() {
            println!(
                "{}",
                serde_json::json!({ "ok": true, "id": speaker.id, "created": false })
            );
        } else {
            println!(
                "{} Speaker already exists: {} (ID: {})",
                "ℹ".blue(),
                speaker.name.bold(),
                speaker.id.dimmed()
            );
        }
        return Ok(());
    }

    // 2. Create if not found
    let sp = ui::spinner("Creating new speaker profile…");
    let create_args = CreateArgs {
        name: args.name,
        email: args.email,
        title: args.title,
        company: args.company,
        ..Default::default()
    };
    let payload = serde_json::to_value(&create_args)?;
    let speaker: Speaker = client.mutate("speaker.admin.create", &payload).await?;
    sp.finish_and_clear();

    if crate::is_agent() {
        println!(
            "{}",
            serde_json::json!({ "ok": true, "id": speaker.id, "created": true })
        );
    } else {
        println!(
            "{} Successfully created speaker {} (ID: {})",
            "✓".green(),
            speaker.name.bold(),
            speaker.id.dimmed()
        );
    }

    Ok(())
}

pub async fn delete(id: &str, yes: bool) -> Result<()> {
    if !yes {
        if !console::Term::stdout().is_term() {
            anyhow::bail!("Confirmation required in non-interactive mode. Pass -y to confirm.");
        }
        let confirmed = dialoguer::Confirm::new()
            .with_prompt(format!("Are you sure you want to delete speaker {id}?"))
            .default(false)
            .interact()?;

        if !confirmed {
            anyhow::bail!("Deletion cancelled.");
        }
    }

    let client = require_client()?;
    client
        .mutate::<serde_json::Value>("speaker.admin.delete", &serde_json::json!({ "id": id }))
        .await?;
    if crate::is_agent() {
        println!("{}", serde_json::json!({ "ok": true, "id": id }));
    } else {
        println!("Successfully deleted speaker {id}.");
    }
    Ok(())
}

pub async fn broadcast(subject: Option<&str>, message: Option<&str>, sync: bool) -> Result<()> {
    let client = require_client()?;

    if sync {
        println!("Syncing speaker list with newsletter audience...");
        let res: serde_json::Value = client
            .mutate("speaker.admin.syncAudience", &serde_json::json!({}))
            .await?;
        println!("Sync response: {res:?}");
    }

    if let (Some(subject), Some(message)) = (subject, message) {
        // Wrap plain text in a basic Portable Text block
        let portable_text = serde_json::json!([{
                "_type": "block",
                "children": [{
                    "_type": "span",
                    "text": message
                }],
                "style": "normal"
        }]);

        client
            .mutate::<serde_json::Value>(
                "speaker.admin.broadcastEmail",
                &serde_json::json!({
                    "subject": subject,
                    "message": portable_text.to_string()
                }),
            )
            .await?;

        if crate::is_agent() {
            println!("{}", serde_json::json!({ "ok": true }));
        } else {
            println!("Broadcast email sent successfully.");
        }
    }

    Ok(())
}

pub async fn sync_audience() -> Result<()> {
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>("speaker.admin.syncAudience", &serde_json::json!({}))
        .await?;

    if crate::is_agent() {
        println!("{}", serde_json::json!({ "ok": true }));
    } else {
        println!("Speaker email audience synced successfully.");
    }
    Ok(())
}
