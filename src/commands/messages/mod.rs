mod args;
pub use args::*;

use anyhow::Result;

use super::require_client;

pub async fn list(args: ListArgs) -> Result<()> {
    let client = require_client()?;
    
    // The endpoint returns { conversations: [...], nextCursor: ... }
    let res: serde_json::Value = client
        .query("message.listConversations", Some(&serde_json::to_value(&args)?))
        .await?;
        
    let conversations = if let Some(arr) = res.as_array() {
        arr.clone()
    } else if let Some(arr) = res.get("conversations").and_then(|v| v.as_array()) {
        arr.clone()
    } else {
        Vec::new()
    };

    if args.json || crate::is_agent() {
        if crate::is_agent() {
            println!("{}", serde_json::to_string(&res)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&res)?);
        }
        return Ok(());
    }

    if conversations.is_empty() {
        println!("No conversations found in view '{:?}'.", args.view);
        return Ok(());
    }

    // TODO: implement a nice table view for conversations
    println!("{}", serde_json::to_string_pretty(&conversations)?);

    if let Some(cursor) = res.get("nextCursor").and_then(|v| v.as_str()) {
        println!("\nNext cursor: {}", cursor);
    }

    Ok(())
}

pub async fn get(id: &str, json: bool) -> Result<()> {
    let client = require_client()?;
    let convo: serde_json::Value = client
        .query("message.getConversation", Some(&serde_json::json!({ "id": id })))
        .await?;
        
    let messages: serde_json::Value = client
        .query("message.listMessages", Some(&serde_json::json!({ "conversationId": id })))
        .await?;

    if json || crate::is_agent() {
        let mut out = convo;
        out["messages"] = messages;
        if crate::is_agent() {
            println!("{}", serde_json::to_string(&out)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        return Ok(());
    }

    // TODO: implement a nice thread view
    println!("{}", serde_json::to_string_pretty(&convo)?);
    println!("{}", serde_json::to_string_pretty(&messages)?);
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
        println!("{}", serde_json::json!({ "ok": true, "id": id, "status": status }));
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
        println!("{}", serde_json::json!({ "ok": true, "id": id, "assignedTo": to }));
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
        println!("{}", serde_json::json!({ "ok": true, "id": id, "archived": archived }));
    } else {
        if archived {
            println!("Conversation {} archived globally.", id);
        } else {
            println!("Conversation {} unarchived globally.", id);
        }
    }
    Ok(())
}
