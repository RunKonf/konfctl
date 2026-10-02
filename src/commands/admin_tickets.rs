use anyhow::Result;

use super::require_client;
use crate::display;
use crate::types::TicketStats;
use crate::ui;

/// Read-only. Every figure comes from `tickets.admin.summary`; this command
/// derives none of them — see `@/lib/tickets/summary` in the website repo.
pub async fn stats(json: bool) -> Result<()> {
    let client = require_client()?;

    let sp = ui::spinner("Fetching ticket figures…");
    let stats: TicketStats = client.query("tickets.admin.summary", None).await?;
    sp.finish_and_clear();

    if json || crate::is_agent() {
        if crate::is_agent() {
            println!("{}", serde_json::to_string(&stats)?);
        } else {
            let raw = serde_json::to_string_pretty(&serde_json::to_value(&stats)?)?;
            println!("{raw}");
        }
    } else {
        display::print_ticket_stats(&stats);
    }

    Ok(())
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportedParticipant {
    first_name: String,
    last_name: String,
    email: String,
    categories: String,
    is_comp: bool,
    grants_workshop: bool,
}

pub async fn participants(workshop_only: bool, json: bool) -> Result<()> {
    let client = require_client()?;
    let sp = ui::spinner("Fetching participants...");

    let payload = serde_json::json!({
        "workshopOnly": workshop_only
    });

    let res: Vec<ExportedParticipant> = client
        .query("tickets.admin.exportParticipants", Some(&payload))
        .await?;
    sp.finish_and_clear();

    if json || crate::is_agent() {
        if crate::is_agent() {
            println!("{}", serde_json::to_string(&res)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&res)?);
        }
    } else {
        println!("FirstName,LastName,Email,Categories,IsComp,GrantsWorkshop");
        for p in res {
            println!(
                "{},{},{},\"{}\",{},{}",
                p.first_name.replace(",", ""),
                p.last_name.replace(",", ""),
                p.email.replace(",", ""),
                p.categories.replace('"', ""),
                p.is_comp,
                p.grants_workshop
            );
        }
    }

    Ok(())
}
