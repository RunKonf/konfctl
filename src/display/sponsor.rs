use std::fmt::Write;

use colored::Colorize;

use crate::types::{SponsorForConference, SponsorStatus};

pub fn print_sponsor_list(sponsors: &[SponsorForConference]) {
    if sponsors.is_empty() {
        println!("No sponsors found.");
        return;
    }

    println!("{SPONSOR_TABLE_HEADER}");

    for s in sponsors {
        println!("{}", format_sponsor_row(s));
    }

    println!("\n{} sponsors", sponsors.len());
}

pub const SPONSOR_TABLE_HEADER: &str =
    "SPONSOR              STATUS         LAST ACT        CONTRACT         TIER";

pub fn format_sponsor_row(s: &SponsorForConference) -> String {
    let name = s.sponsor.as_ref().map_or("Unknown", |sp| sp.name.as_str());
    let tier = s.tier.as_ref().map_or("-", |t| t.title.as_str());
    let contract = s.contract_status.as_deref().unwrap_or("-");

    // Pad the status text *before* colorizing so ANSI codes don't break alignment
    let status_padded = format!("{:<14}", s.status);
    let status_colored = colorize_status_str(&status_padded, s.status);

    let last_act = s.last_activity.as_ref().map_or("-".to_string(), |a| {
        format!("{:.10}", a.created_at) // YYYY-MM-DD
    });

    format!(
        "{:<20} {} {:<15} {:<16} {}",
        truncate(name, 18),
        status_colored,
        last_act.dimmed(),
        contract,
        tier
    )
}

/// Render sponsor details into a `String` (for scrollable views, etc.).
#[allow(clippy::too_many_lines)]
pub fn render_sponsor_detail(sponsor: &SponsorForConference) -> String {
    let mut buf = String::new();
    let name = sponsor
        .sponsor
        .as_ref()
        .map_or("Unknown", |s| s.name.as_str());

    writeln!(buf, "{}", name.bold()).unwrap();
    writeln!(buf, "ID:              {}", sponsor.id).unwrap();
    writeln!(buf, "Status:          {}", colorize_status(sponsor.status)).unwrap();
    if let Some(contract) = &sponsor.contract_status {
        writeln!(buf, "Contract:        {contract}").unwrap();
    }
    if let Some(invoice) = &sponsor.invoice_status {
        writeln!(buf, "Invoice:         {invoice}").unwrap();
    }
    if let Some(tier) = &sponsor.tier {
        writeln!(buf, "Tier:            {}", tier.title).unwrap();
    }
    if let Some(assigned) = &sponsor.assigned_to {
        writeln!(buf, "Assigned to:     {}", assigned.name).unwrap();
    }
    if let Some(value) = sponsor.contract_value {
        let currency = sponsor.contract_currency.as_deref().unwrap_or("NOK");
        writeln!(buf, "Contract value:  {value} {currency}").unwrap();
    }
    if let Some(website) = sponsor.sponsor.as_ref().and_then(|s| s.website.as_deref()) {
        writeln!(buf, "Website:         {website}").unwrap();
    }
    if let Some(linkedin) = sponsor
        .sponsor
        .as_ref()
        .and_then(|s| s.linkedin_url.as_deref())
    {
        writeln!(buf, "LinkedIn (Org):  {linkedin}").unwrap();
    }
    if let Some(follow_up) = &sponsor.next_follow_up_at {
        writeln!(buf, "Next Follow-up:  {}", follow_up.yellow()).unwrap();
    }
    if let Some(outreach) = sponsor.outreach_count {
        writeln!(buf, "Outreach Count:  {outreach}").unwrap();
    }

    if !sponsor.contact_persons.is_empty() {
        writeln!(buf, "\nContacts:").unwrap();
        for c in &sponsor.contact_persons {
            let role = c.role.as_deref().unwrap_or("");
            let email = c.email.as_deref().unwrap_or("");
            let primary = if c.is_primary.unwrap_or(false) {
                " [primary]"
            } else {
                ""
            };
            let linkedin = c
                .linkedin_url
                .as_ref()
                .map(|url| format!(" | LinkedIn: {url}"))
                .unwrap_or_default();
            writeln!(
                buf,
                "  - {} <{}> {}{}{}",
                c.name, email, role, primary, linkedin
            )
            .unwrap();
        }
    }

    if let Some(billing) = &sponsor.billing
        && (billing.email.is_some() || billing.reference.is_some())
    {
        writeln!(buf, "\nBilling:").unwrap();
        if let Some(email) = &billing.email {
            writeln!(buf, "  Email:     {email}").unwrap();
        }
        if let Some(reference) = &billing.reference {
            writeln!(buf, "  Reference: {reference}").unwrap();
        }
    }

    if let Some(notes) = &sponsor.notes
        && !notes.is_empty()
    {
        writeln!(buf, "\nNotes:\n{notes}").unwrap();
    }

    if !sponsor.tags.is_empty() {
        writeln!(buf, "\nTags: {}", sponsor.tags.join(", ")).unwrap();
    }

    if let Some(count) = sponsor.activity_count {
        writeln!(buf, "Total activities: {count}").unwrap();
    }

    if let Some(last) = &sponsor.last_activity {
        writeln!(buf, "\nLast Activity:").unwrap();
        let date = &last.created_at[..10];
        let type_label = format!("[{}]", last.kind).to_uppercase();
        let author = last
            .created_by
            .as_ref()
            .map(|a| format!(" by {}", a.name))
            .unwrap_or_default();
        writeln!(
            buf,
            "  {} {} {}{}",
            date.dimmed(),
            type_label.yellow(),
            last.description,
            author.dimmed()
        )
        .unwrap();
    }

    if !sponsor.activities.is_empty() {
        writeln!(buf, "\nRecent History:").unwrap();
        for activity in sponsor.activities.iter().take(5) {
            let date = &activity.created_at[..10];
            let type_label = format!("[{}]", activity.activity_type).to_uppercase();
            writeln!(
                buf,
                "  {} {} {}",
                date.dimmed(),
                type_label.yellow(),
                activity.description
            )
            .unwrap();
        }
        if sponsor.activities.len() > 5 {
            writeln!(buf, "  ... (use `history` for full log)").unwrap();
        }
    }

    buf
}

/// Print sponsor details to stdout.
pub fn print_sponsor_detail(sponsor: &SponsorForConference) {
    print!("{}", render_sponsor_detail(sponsor));
}

pub fn print_sponsor_history(sponsor: &SponsorForConference) {
    let name = sponsor
        .sponsor
        .as_ref()
        .map_or("Unknown", |s| s.name.as_str());

    println!("{} - History", name.bold());
    println!("ID: {}\n", sponsor.id);

    println!(
        "{}",
        "Hint: Log new interactions with `konf admin sponsors note <ID> \"...\"`".dimmed()
    );
    println!();

    if sponsor.activities.is_empty() {
        println!("No activities recorded yet.");
        return;
    }

    for activity in &sponsor.activities {
        let date = &activity.created_at[..10]; // Simple YYYY-MM-DD
        let type_label = format!("[{}]", activity.activity_type).to_uppercase();
        let author = activity
            .created_by
            .as_ref()
            .map(|a| format!(" by {}", a.name))
            .unwrap_or_default();

        println!(
            "{} {} {}{}",
            date.dimmed(),
            type_label.yellow(),
            activity.description,
            author.dimmed()
        );
    }
}

fn colorize_status(status: SponsorStatus) -> String {
    colorize_status_str(&status.to_string(), status)
}

fn colorize_status_str(label: &str, status: SponsorStatus) -> String {
    match status {
        SponsorStatus::ClosedWon => label.green().to_string(),
        SponsorStatus::Negotiating => label.yellow().to_string(),
        SponsorStatus::Contacted => label.cyan().to_string(),
        SponsorStatus::Prospect | SponsorStatus::Unknown => label.dimmed().to_string(),
        SponsorStatus::ClosedLost => label.red().to_string(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    crate::ui::truncate(s, max)
}
