#![allow(
    clippy::if_not_else,
    clippy::collapsible_else_if,
    clippy::uninlined_format_args,
    clippy::nonminimal_bool,
    clippy::needless_bool
)]
mod args;
pub use args::*;

use anyhow::Result;
use colored::Colorize;

use super::require_client;
// noop
use crate::types::{ConversationRow, GetConversationResult};

mod interactive;

pub async fn list(args: ListArgs) -> Result<()> {
    let client = require_client()?;

    // The endpoint returns { conversations: [...], nextCursor: ... }
    let res: serde_json::Value = client
        .query(
            "message.listConversations",
            Some(&serde_json::to_value(&args)?),
        )
        .await?;

    if args.json && !crate::is_agent() {
        println!("{}", serde_json::to_string_pretty(&res)?);
        return Ok(());
    }

    let conversations: Vec<ConversationRow> = if let Some(arr) = res.as_array() {
        serde_json::from_value(serde_json::Value::Array(arr.clone()))?
    } else if let Some(arr) = res.get("conversations").and_then(|v| v.as_array()) {
        serde_json::from_value(serde_json::Value::Array(arr.clone()))?
    } else {
        Vec::new()
    };

    let mut conversations = conversations;
    conversations.sort_by(|a, b| b.last_message_at.cmp(&a.last_message_at));

    if args.json || crate::is_agent() {
        let out_value = if args.compact {
            serde_json::to_value(
                conversations
                    .into_iter()
                    .take(args.limit.unwrap_or(usize::MAX))
                    .map(|c| {
                        serde_json::json!({
                            "id": c.id,
                            "subject": c.subject,
                            "counterpart": c.counterpart.as_ref().map(|cp| &cp.name),
                            "status": c.status,
                            "unread": c.unread_count,
                            "proposalId": c.proposal_id,
                        })
                    })
                    .collect::<Vec<_>>(),
            )?
        } else {
            res
        };

        if crate::is_agent() {
            println!("{}", serde_json::to_string(&out_value)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&out_value)?);
        }
        return Ok(());
    }

    if conversations.is_empty() {
        println!("No conversations found in view '{:?}'.", args.view);
        return Ok(());
    }

    if !console::Term::stdout().is_term() {
        // Just print the rows non-interactively
        println!(
            "{:<25} | {:<50} | {:<20} | {}",
            "ID".bold().cyan(),
            "SUBJECT".bold().cyan(),
            "COUNTERPART".bold().cyan(),
            "STATUS".bold().cyan()
        );
        for convo in &conversations {
            println!("{}", interactive::format_item(convo));
        }
    } else {
        interactive::run(&client, conversations).await?;
    }

    Ok(())
}

pub async fn get(id: &str, json: bool) -> Result<()> {
    let client = require_client()?;
    let convo: GetConversationResult = client
        .query(
            "message.getConversation",
            Some(&serde_json::json!({ "id": id })),
        )
        .await?;

    let messages: Vec<crate::types::ConversationMessage> = client
        .query(
            "message.listMessages",
            Some(&serde_json::json!({ "conversationId": id })),
        )
        .await?;

    if json || crate::is_agent() {
        let mut out = serde_json::to_value(&convo)?;
        out["messages"] = serde_json::to_value(&messages)?;
        if crate::is_agent() {
            println!("{}", serde_json::to_string(&out)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        return Ok(());
    }

    let subject_raw = convo.conversation.subject.as_deref().unwrap_or("No subject");
    let subject_safe = console::strip_ansi_codes(subject_raw);
    
    println!(
        "{} {}\n{}",
        "Thread:".bold().cyan(),
        subject_safe.bold(),
        format!("ID: {}", convo.conversation.id).dimmed()
    );

    println!("{}", "Participants:".bold().cyan());
    for p in &convo.participants {
        let name_raw = p.name.as_deref().unwrap_or("Unknown");
        let name_safe = console::strip_ansi_codes(name_raw);
        println!(
            "  - {} {}",
            name_safe,
            if p.is_organizer {
                "(Organizer)".dimmed()
            } else {
                "".dimmed()
            }
        );
    }

    println!("\n{}\n", "Messages:".bold().cyan());
    for msg in messages.iter().rev() {
        let author_raw = msg.author_name.as_deref().unwrap_or("Unknown");
        let author_safe = console::strip_ansi_codes(author_raw);
        let date = msg.created_at.as_str();
        println!("{} [{}]", author_safe.bold().blue(), date.dimmed());
        
        let body_safe = console::strip_ansi_codes(&msg.body);
        println!("{}\n", body_safe);
    }

    Ok(())
}

pub async fn reply(id: &str, message: &str) -> Result<()> {
    let client = require_client()?;

    let _res: serde_json::Value = client
        .mutate(
            "message.send",
            &serde_json::json!({
                "conversationId": id,
                "body": message,
            }),
        )
        .await?;

    if crate::is_agent() {
        println!("{}", serde_json::json!({ "ok": true, "id": id }));
    } else {
        println!("Reply sent successfully.");
    }
    Ok(())
}

pub async fn start_new(speaker_id: &str, subject: &str, message: &str) -> Result<()> {
    let client = require_client()?;

    let _res: serde_json::Value = client
        .mutate(
            "message.send",
            &serde_json::json!({
                "subject": subject,
                "recipientSpeakerId": speaker_id,
                "body": message,
            }),
        )
        .await?;

    if crate::is_agent() {
        println!("{}", serde_json::json!({ "ok": true }));
    } else {
        println!("New conversation started and message sent successfully.");
    }
    Ok(())
}

pub async fn set_status(id: &str, status: ConversationStatusEnum) -> Result<()> {
    let client = require_client()?;

    client
        .mutate::<serde_json::Value>(
            "message.setStatus",
            &serde_json::json!({
                "conversationId": id,
                "status": status,
            }),
        )
        .await?;

    if crate::is_agent() {
        println!(
            "{}",
            serde_json::json!({ "ok": true, "id": id, "status": status })
        );
    } else {
        println!("Conversation {} status set to {:?}.", id, status);
    }
    Ok(())
}

pub async fn set_assignee(id: &str, to: Option<&str>) -> Result<()> {
    let client = require_client()?;

    client
        .mutate::<serde_json::Value>(
            "message.setAssignee",
            &serde_json::json!({
                "conversationId": id,
                "assigneeId": to,
            }),
        )
        .await?;

    if crate::is_agent() {
        println!(
            "{}",
            serde_json::json!({ "ok": true, "id": id, "assignedTo": to })
        );
    } else {
        if let Some(user_id) = to {
            println!("Conversation {} assigned to {}.", id, user_id);
        } else {
            println!("Conversation {} unassigned.", id);
        }
    }
    Ok(())
}

pub async fn set_archive(id: &str, unarchive: bool) -> Result<()> {
    let client = require_client()?;
    let archived = !unarchive;

    client
        .mutate::<serde_json::Value>(
            "message.setArchived",
            &serde_json::json!({
                "conversationId": id,
                "archived": archived,
            }),
        )
        .await?;

    if crate::is_agent() {
        println!(
            "{}",
            serde_json::json!({ "ok": true, "id": id, "archived": archived })
        );
    } else {
        if archived {
            println!("Conversation {} archived globally.", id);
        } else {
            println!("Conversation {} unarchived globally.", id);
        }
    }
    Ok(())
}
