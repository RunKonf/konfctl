mod args;
mod display;
mod interactive;
mod review;

#[cfg(test)]
mod tests;

pub use args::{ActionArgs, CreateArgs, DeleteArgs, ListArgs, ReviewArgs, UpdateArgs};

use anyhow::Result;

use super::require_client;
use crate::client::TrpcClient;
use crate::types::{Proposal, ReviewInput};
use crate::ui;

// ── API helpers ──────────────────────────────────────────────────────────────

pub async fn fetch_all(client: &TrpcClient, args: &ListArgs) -> Result<Vec<Proposal>> {
    let mut payload = args.clone();
    if payload.unreviewed {
        payload.review_status = Some(crate::types::ReviewStatus::Unreviewed);
    }
    client
        .query(
            "proposal.admin.list",
            Some(&serde_json::to_value(&payload)?),
        )
        .await
}

pub async fn fetch_one(client: &TrpcClient, id: &str) -> Result<Proposal> {
    let input = serde_json::json!({ "id": id });
    client.query("proposal.admin.getById", Some(&input)).await
}

pub async fn submit_review(client: &TrpcClient, input: &ReviewInput) -> Result<serde_json::Value> {
    client
        .mutate("proposal.admin.submitReview", &serde_json::to_value(input)?)
        .await
}

pub async fn add(args: CreateArgs) -> Result<()> {
    let client = require_client()?;

    if args.title.is_empty() && console::Term::stdout().is_term() {
        return interactive::add_wizard(&client).await;
    }

    let mut payload = serde_json::to_value(&args)?;

    // Wrap plain text description in a Portable Text array
    if let Some(desc) = args.description {
        let portable_text = serde_json::json!([{
            "_type": "block",
            "children": [{
                "_type": "span",
                "text": desc
            }],
            "style": "normal"
        }]);
        payload["description"] = portable_text;
    }

    // Convert topic IDs to Sanity references
    if let Some(topics) = args.topics {
        let refs: Vec<serde_json::Value> = topics
            .into_iter()
            .map(|id| serde_json::json!({ "_type": "reference", "_ref": id }))
            .collect();
        payload["topics"] = serde_json::json!(refs);
    }

    let proposal: Proposal = client.mutate("proposal.admin.create", &payload).await?;

    if crate::is_agent() {
        println!(
            "{}",
            serde_json::json!({ "ok": true, "id": proposal.id, "title": proposal.title })
        );
    } else {
        println!(
            "Successfully created proposal {} (ID: {})",
            proposal.title, proposal.id
        );
    }
    Ok(())
}

pub async fn delete(args: DeleteArgs) -> Result<()> {
    if !args.yes && console::Term::stdout().is_term() {
        let confirmed = dialoguer::Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete proposal {}?",
                args.id
            ))
            .default(false)
            .interact()?;

        if !confirmed {
            anyhow::bail!("Deletion cancelled.");
        }
    }

    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "proposal.admin.delete",
            &serde_json::json!({ "id": args.id }),
        )
        .await?;

    if crate::is_agent() {
        println!("{}", serde_json::json!({ "ok": true, "id": args.id }));
    } else {
        println!("Successfully deleted proposal {}.", args.id);
    }
    Ok(())
}

pub async fn action(args: ActionArgs) -> Result<()> {
    let client = require_client()?;
    let res: serde_json::Value = client
        .mutate(
            "proposal.action",
            &serde_json::json!({
                "id": args.id,
                "action": args.action,
                "notify": args.notify,
                "comment": args.comment,
            }),
        )
        .await?;

    let status = res
        .get("proposalStatus")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown");

    if crate::is_agent() {
        println!(
            "{}",
            serde_json::json!({ "ok": true, "id": args.id, "status": status })
        );
    } else {
        println!("Action performed successfully. New status: {status}");
    }
    Ok(())
}

pub async fn update(args: UpdateArgs) -> Result<()> {
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "proposal.admin.update",
            &serde_json::json!({
                "id": args.id,
                "data": args,
            }),
        )
        .await?;

    if crate::is_agent() {
        println!("{}", serde_json::json!({ "ok": true, "id": args.id }));
    } else {
        println!("Successfully updated proposal {}.", args.id);
    }
    Ok(())
}

pub async fn add_speaker(proposal_id: &str, speaker_query: &str) -> Result<()> {
    let client = require_client()?;

    // 1. Identify the speaker (ID or Email search)
    let speaker_id = if speaker_query.contains('@') {
        let sp = ui::spinner("Searching for speaker…");
        let search_args = crate::commands::speakers::ListArgs {
            query: Some(speaker_query.to_string()),
            all: true,
            ..Default::default()
        };
        let results = crate::commands::speakers::fetch_search(&client, &search_args).await?;
        sp.finish_and_clear();

        results
            .first()
            .map(|s| s.id.clone())
            .ok_or_else(|| anyhow::anyhow!("Speaker not found for email: {speaker_query}"))?
    } else {
        speaker_query.to_string()
    };

    // 2. Get current proposal
    let sp = ui::spinner("Fetching current proposal…");
    let proposal = fetch_one(&client, proposal_id).await?;
    sp.finish_and_clear();

    // 3. Update with new speaker list
    let mut speaker_ids: Vec<String> = proposal.speakers.into_iter().map(|s| s.id).collect();
    if speaker_ids.contains(&speaker_id) {
        println!("Speaker is already associated with this proposal.");
        return Ok(());
    }
    speaker_ids.push(speaker_id.clone());

    let update_args = UpdateArgs {
        id: proposal_id.to_string(),
        speakers: Some(speaker_ids),
        ..Default::default()
    };

    update(update_args).await?;
    if crate::is_agent() {
        println!(
            "{}",
            serde_json::json!({ "ok": true, "proposal_id": proposal_id, "speaker_id": speaker_id })
        );
    } else {
        println!("Added speaker {speaker_id} to proposal {proposal_id}.");
    }

    Ok(())
}

pub async fn next_review() -> Result<()> {
    let client = require_client()?;
    let reviewer_name = crate::config::load().ok().and_then(|c| c.name);

    let sp = ui::spinner("Fetching next unreviewed proposal…");
    let input = serde_json::json!({});
    let proposal_opt: Option<Proposal> = client
        .query("proposal.admin.nextUnreviewed", Some(&input))
        .await?;
    sp.finish_and_clear();

    match proposal_opt {
        Some(proposal) => {
            crate::display::print_proposal_detail(&proposal);
            println!();
            review::prompt_and_submit_review(&client, &proposal, reviewer_name.as_deref()).await?;
        }
        None => {
            println!("No unreviewed proposals found. Great job!");
        }
    }

    Ok(())
}

// ── Command entry points ─────────────────────────────────────────────────────

pub async fn list(args: ListArgs) -> Result<()> {
    let client = require_client()?;

    let sp = ui::spinner("Fetching proposals…");
    let all = fetch_all(&client, &args).await?;
    sp.finish_and_clear();

    let unhandled =
        crate::display::print_json_list(all, args.limit, args.compact, args.json, |p| {
            serde_json::json!({
                "id": p.id,
                "title": p.title,
                "status": p.status,
                "speakers": p.speakers.iter().map(|s| &s.name).collect::<Vec<_>>()
            })
        })?;

    if let Some(all) = unhandled {
        if args.has_cli_filters() || !console::Term::stdout().is_term() {
            if all.is_empty() {
                println!("No proposals match the given filters.");
                return Ok(());
            }

            println!("{}", display::TABLE_HEADER);
            for p in &all {
                println!("{}", display::format_item(p));
            }
        } else {
            interactive::list_interactive(&client, &all).await?;
        }
    }
    Ok(())
}

pub async fn get(id: &str, json: bool) -> Result<()> {
    let client = require_client()?;

    let sp = ui::spinner("Fetching proposal…");
    let proposal = fetch_one(&client, id).await?;
    sp.finish_and_clear();

    if json || crate::is_agent() {
        if crate::is_agent() {
            println!("{}", serde_json::to_string(&proposal)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&proposal)?);
        }
    } else {
        crate::display::print_proposal_detail(&proposal);
    }
    Ok(())
}

pub async fn review(args: ReviewArgs) -> Result<()> {
    use crate::types::ReviewScore;
    use crate::{config, display};

    let client = require_client()?;
    let reviewer_name = config::load().ok().and_then(|c| c.name);

    let sp = ui::spinner("Fetching proposal…");
    let proposal = fetch_one(&client, &args.id).await?;
    sp.finish_and_clear();

    display::print_proposal_detail(&proposal);
    println!();

    // If all scores and comment are provided, submit non-interactively
    if let (Some(content), Some(relevance), Some(speaker), Some(comment)) =
        (args.content, args.relevance, args.speaker, args.comment)
    {
        let input = ReviewInput {
            id: args.id,
            comment,
            score: ReviewScore {
                content: f64::from(content),
                relevance: f64::from(relevance),
                speaker: f64::from(speaker),
            },
        };

        let sp = ui::spinner("Submitting review…");
        submit_review(&client, &input).await?;
        sp.finish_and_clear();

        println!("Review submitted ({:.0}/15)", input.score.total());
    } else {
        review::prompt_and_submit_review(&client, &proposal, reviewer_name.as_deref()).await?;
    }

    Ok(())
}
