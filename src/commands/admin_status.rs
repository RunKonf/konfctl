use anyhow::Result;

use super::require_client;
use crate::display;
use crate::types::ConferenceStatusSummary;
use crate::ui;

pub async fn run(json: bool) -> Result<()> {
    let client = require_client()?;

    let sp = ui::spinner("Fetching conference status…");
    let summary: ConferenceStatusSummary = client.query("status.admin.summary", None).await?;
    sp.finish_and_clear();

    if json || crate::is_agent() {
        if crate::is_agent() {
            println!("{}", serde_json::to_string(&summary)?);
        } else {
            let raw = serde_json::to_string_pretty(&serde_json::to_value(&summary)?)?;
            println!("{raw}");
        }
    } else {
        display::print_status(&summary);
    }

    Ok(())
}
