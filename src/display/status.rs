use std::fmt::Write;

use colored::Colorize;

use crate::types::ConferenceStatusSummary;

pub fn print_status(summary: &ConferenceStatusSummary) {
    print!("{}", render_status(summary));
}

fn render_status(summary: &ConferenceStatusSummary) -> String {
    let mut buf = String::new();

    let date = summary
        .last_updated
        .split('T')
        .next()
        .unwrap_or(&summary.last_updated);

    writeln!(
        buf,
        "{}",
        format!("📊 Conference Status — {}", summary.conference_title).bold()
    )
    .unwrap();
    writeln!(buf, "   Summary as of {date}").unwrap();

    if let Some(sp) = &summary.sponsors {
        render_sponsors(&mut buf, sp);
    }
    if let Some(pr) = &summary.proposals {
        render_proposals(&mut buf, pr);
    }
    if let Some(tk) = &summary.tickets {
        render_tickets(&mut buf, tk);
    }
    if let Some(tp) = &summary.target_progress {
        render_target_progress(&mut buf, tp, summary.tickets.as_ref());
    }
    if let Some(tk) = &summary.tickets
        && tk.category_breakdown.len() > 1
    {
        render_category_breakdown(&mut buf, tk);
    }
    if !summary.errors.is_empty() {
        render_errors(&mut buf, &summary.errors);
    }

    render_hints(&mut buf, summary);

    buf
}

fn render_hints(buf: &mut String, summary: &ConferenceStatusSummary) {
    writeln!(buf).unwrap();
    writeln!(buf, "{}", "💡 Next Steps for Agents:".dimmed()).unwrap();

    if let Some(pr) = &summary.proposals
        && pr.submitted > 0
    {
        writeln!(
            buf,
            "{}",
            "• There are unreviewed proposals. Use `konf admin proposals list` to start reviewing."
                .dimmed()
        )
        .unwrap();
    }

    if let Some(sp) = &summary.sponsors
        && sp.active_deals > 0
    {
        writeln!(
            buf,
            "{}",
            "• You have active sponsor deals. Use `konf admin sponsors list` to check progress."
                .dimmed()
        )
        .unwrap();
    }

    writeln!(
        buf,
        "{}",
        "• Need more context? Run `konf agents get` for conference rules and goals.".dimmed()
    )
    .unwrap();
}

fn render_sponsors(buf: &mut String, sp: &crate::types::SponsorPipeline) {
    writeln!(buf).unwrap();
    writeln!(buf, "{}", "🤝 Sponsor Pipeline".bold()).unwrap();
    write_row(
        buf,
        "Total Sponsors:",
        sp.total_sponsors,
        "Active Deals:",
        sp.active_deals,
    );
    write_row(
        buf,
        "Closed Won:",
        sp.closed_won_count,
        "Closed Lost:",
        sp.closed_lost_count,
    );

    if sp.total_contract_value > 0.0 {
        let closed = sp.closed_won_count + sp.closed_lost_count;
        let win_rate = if closed > 0 {
            #[allow(clippy::cast_precision_loss)]
            let rate = sp.closed_won_count as f64 / closed as f64 * 100.0;
            format!("{rate:.0}%")
        } else {
            "0%".to_string()
        };
        writeln!(
            buf,
            "  {:<24}{:<6}  {:<24}{win_rate}",
            "Total Contract Value:",
            format_currency(sp.total_contract_value, &sp.contract_currency),
            "Win Rate:",
        )
        .unwrap();
    }

    for (label, map) in [
        ("Pipeline Stages:", &sp.by_status),
        ("Invoice Status:", &sp.by_invoice_status),
        ("Contract Status:", &sp.by_contract_status),
    ] {
        let text = format_map(map);
        if !text.is_empty() {
            writeln!(buf, "  {label:<19}{text}").unwrap();
        }
    }
}

fn render_proposals(buf: &mut String, pr: &crate::types::ProposalSummary) {
    writeln!(buf).unwrap();
    writeln!(buf, "{}", "📝 CFP / Proposals".bold()).unwrap();
    write_row(
        buf,
        "Total Proposals:",
        pr.total,
        "Submitted:",
        pr.submitted,
    );
    write_row(buf, "Accepted:", pr.accepted, "Confirmed:", pr.confirmed);
    if pr.rejected > 0 || pr.withdrawn > 0 {
        write_row(buf, "Rejected:", pr.rejected, "Withdrawn:", pr.withdrawn);
    }
}

fn render_tickets(buf: &mut String, tk: &crate::types::TicketSummary) {
    writeln!(buf).unwrap();
    writeln!(buf, "{}", "🎟️  Tickets".bold()).unwrap();
    writeln!(
        buf,
        "  {:<24}{:<6}  {:<24}{}",
        "Paid Tickets:",
        tk.paid_tickets,
        "Total Revenue:",
        format_currency(tk.total_revenue, "kr")
    )
    .unwrap();

    let complimentary = tk.sponsor_tickets + tk.speaker_tickets + tk.organizer_tickets;
    let comp_detail = format!(
        "{complimentary} (claimed {}, rate {:.1}%)",
        tk.free_tickets_claimed, tk.free_ticket_claim_rate
    );
    writeln!(
        buf,
        "  {:<24}{:<6}  {:<24}{comp_detail}",
        "Total Tickets:", tk.total_tickets, "Complimentary:",
    )
    .unwrap();
}

fn render_target_progress(
    buf: &mut String,
    tp: &crate::types::TargetProgress,
    tickets: Option<&crate::types::TicketSummary>,
) {
    writeln!(buf).unwrap();
    let emoji = if tp.is_on_track { "✅" } else { "⚠️" };
    writeln!(buf, "{}", format!("{emoji} Target Progress").bold()).unwrap();

    let target = format!("{:.1}%", tp.target_percentage);
    writeln!(
        buf,
        "  {:<24}{:<6}  {:<24}{:.1}%",
        "Current Target:", target, "Actual Progress:", tp.current_percentage,
    )
    .unwrap();

    let variance_text = if tp.variance >= 0.0 {
        format!("+{:.1}% ahead", tp.variance).green().to_string()
    } else {
        format!("{:.1}% behind", tp.variance).red().to_string()
    };
    if let Some(tk) = tickets {
        writeln!(
            buf,
            "  {:<24}{:<30}  {:<24}{}/{}",
            "Variance:", variance_text, "Capacity:", tk.paid_tickets, tp.capacity
        )
        .unwrap();
    } else {
        writeln!(buf, "  {:<24}{variance_text}", "Variance:").unwrap();
    }

    if let Some(m) = &tp.next_milestone {
        writeln!(
            buf,
            "  🎯 Next Milestone: {} in {} days",
            m.label, m.days_away
        )
        .unwrap();
    }
}

fn render_category_breakdown(buf: &mut String, tk: &crate::types::TicketSummary) {
    writeln!(buf).unwrap();
    writeln!(buf, "{}", "Breakdown by Paid Ticket Category:".bold()).unwrap();
    let mut entries: Vec<_> = tk.category_breakdown.iter().collect();
    entries.sort_by(|a, b| b.1.cmp(a.1));
    for (cat, count) in entries {
        writeln!(buf, "  {cat}: {count} tickets").unwrap();
    }
}

fn render_errors(buf: &mut String, errors: &[crate::types::SectionError]) {
    writeln!(buf).unwrap();
    for e in errors {
        writeln!(
            buf,
            "{}",
            format!("⚠ {}: {}", e.section, e.message).yellow()
        )
        .unwrap();
    }
}

fn write_row(buf: &mut String, l1: &str, v1: usize, l2: &str, v2: usize) {
    writeln!(buf, "  {l1:<24}{v1:<6}  {l2:<24}{v2}").unwrap();
}

#[allow(clippy::cast_possible_truncation)]
fn format_currency(amount: f64, currency: &str) -> String {
    let integer = amount as i64;
    let formatted = format_thousands(integer);
    format!("{formatted} {currency}")
}

fn format_thousands(n: i64) -> String {
    let s = n.to_string();
    let bytes: Vec<u8> = s.bytes().rev().collect();
    let chunks: Vec<String> = bytes
        .chunks(3)
        .map(|c| c.iter().rev().map(|&b| b as char).collect())
        .collect();
    let mut result: Vec<String> = chunks;
    result.reverse();
    result.join("\u{00a0}")
}

fn format_map(map: &std::collections::HashMap<String, usize>) -> String {
    let mut entries: Vec<_> = map.iter().filter(|(_, v)| **v > 0).collect();
    entries.sort_by(|a, b| b.1.cmp(a.1));
    entries
        .iter()
        .map(|(k, v)| format!("{k}: {v}"))
        .collect::<Vec<_>>()
        .join(" · ")
}
