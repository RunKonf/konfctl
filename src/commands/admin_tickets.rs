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
