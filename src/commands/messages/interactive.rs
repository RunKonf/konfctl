use anyhow::Result;
use colored::Colorize;
use dialoguer::FuzzySelect;

use crate::client::TrpcClient;
use crate::types::{ConversationRow, GetConversationResult};
use crate::ui;

// noop

pub fn format_item(convo: &ConversationRow) -> String {
    let subject = convo.subject.as_deref().unwrap_or("No subject");
    let counterpart = convo.counterpart.as_ref().map(|c| c.name.as_str()).unwrap_or("Unknown");
    
    let status = if convo.status == "resolved" {
        "Resolved".green()
    } else if convo.needs_reply {
        "Needs Reply".red()
    } else {
        "Open".yellow()
    };
    
    let unread = if convo.unread_count > 0 {
        format!("{} unread", convo.unread_count).red().bold().to_string()
    } else {
        "".to_string()
    };
    
    let mut subject_display = subject.to_string();
    if convo.unread_count > 0 {
        subject_display = subject_display.bold().to_string();
    }
    
    // Truncate strings to keep it clean
    let subject_trunc = ui::truncate(&subject_display, 50);
    let counterpart_trunc = ui::truncate(counterpart, 20);
    
    format!(
        "{:<25} | {:<50} | {:<20} | {:<12} {}",
        convo.id,
        subject_trunc,
        counterpart_trunc,
        status,
        unread
    )
}

pub async fn run(client: &TrpcClient, conversations: Vec<ConversationRow>) -> Result<()> {
    if conversations.is_empty() {
        println!("No conversations found.");
        return Ok(());
    }

    let hints = "↑↓ navigate · type to search · enter select · esc quit".dimmed();
    let header = format!(
        "{:<25} | {:<50} | {:<20} | {}",
        "ID".bold().cyan(),
        "SUBJECT".bold().cyan(),
        "COUNTERPART".bold().cyan(),
        "STATUS".bold().cyan()
    );

    let mut cursor = 0usize;

    loop {
        let items: Vec<String> = conversations.iter().map(format_item).collect();
        let max_rows = ui::max_visible_items(&items, 4);

        let selection = FuzzySelect::new()
            .with_prompt(format!(
                "{} conversations\n  {header}\n  {hints}",
                conversations.len()
            ))
            .items(&items)
            .default(cursor)
            .max_length(max_rows)
            .highlight_matches(false)
            .interact_opt()?;

        let Some(idx) = selection else {
            // esc / cancel
            break;
        };

        cursor = idx;
        let selected = &conversations[idx];

        println!("\nLoading thread for {}...\n", selected.id);
        if let Err(e) = view_thread(client, &selected.id).await {
            println!("{} Failed to load thread: {}", "Error".red(), e);
        }
        
        println!("\nPress Enter to return to inbox...");
        let _ = console::Term::stdout().read_key();
    }

    Ok(())
}

async fn view_thread(client: &TrpcClient, id: &str) -> Result<()> {
    // 1. Fetch thread
    let convo: GetConversationResult = client
        .query("message.getConversation", Some(&serde_json::json!({ "id": id })))
        .await?;
        
    let messages: Vec<crate::types::ConversationMessage> = client
        .query("message.listMessages", Some(&serde_json::json!({ "conversationId": id })))
        .await?;

    // 2. Print thread
    println!("{} {}\n{}", "Thread:".bold().cyan(), convo.conversation.subject.as_deref().unwrap_or("No subject").bold(), format!("ID: {}", convo.conversation.id).dimmed());

    println!("{}", "Participants:".bold().cyan());
    for p in &convo.participants {
        println!("  - {} {}", p.name.as_deref().unwrap_or("Unknown"), if p.is_organizer { "(Organizer)".dimmed() } else { "".dimmed() });
    }
    
    println!("\n{}\n", "Messages:".bold().cyan());
    for msg in messages.iter().rev() {
        let author = msg.author_name.as_deref().unwrap_or("Unknown");
        let date = msg.created_at.as_str(); // Format date nicely if needed
        println!("{} [{}]", author.bold().blue(), date.dimmed());
        println!("{}\n", msg.body);
    }
    
    println!("{}", "End of thread".dimmed());
    Ok(())
}
