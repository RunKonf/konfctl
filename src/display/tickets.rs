//! Renders `tickets.admin.summary`. It computes NOTHING.
//!
//! Every number below is printed exactly as the procedure answered it. Where
//! the server said it does not know — an `'unknown'` claim, a failed analysis,
//! a headcount resting on proposed ticket-type roles — this says so instead of
//! printing a confident zero.

use std::fmt::Write;

use colored::Colorize;

use super::status::format_thousands;
use crate::types::{
    AnalysisOutcome, FreeAllocationCategory, FreeTicketAllocation, ParticipantTally, RoleBasis,
    TicketStats, TicketStatsReady, TicketingAccessState,
};

pub fn print_ticket_stats(stats: &TicketStats) {
    print!("{}", render_ticket_stats(stats));
}

fn render_ticket_stats(stats: &TicketStats) -> String {
    let mut buf = String::new();
    match stats {
        TicketStats::Ready(r) => render_ready(&mut buf, r),
        TicketStats::Unconfigured(a) => {
            render_not_ready(
                &mut buf,
                a,
                "Ticketing is not configured for this conference.",
            );
        }
        TicketStats::Unavailable(a) => render_not_ready(
            &mut buf,
            a,
            "Ticketing is configured but unavailable — check the provider credentials.",
        ),
        TicketStats::Disabled(a) => {
            render_not_ready(&mut buf, a, "Ticketing is disabled for this conference.");
        }
        // A FAILED read. Deliberately not rendered as an empty event.
        TicketStats::Error { message } => {
            writeln!(buf, "{}", "🎟️  Tickets".bold()).unwrap();
            writeln!(
                buf,
                "{}",
                format!("⚠ Ticket figures unavailable: {message}").yellow()
            )
            .unwrap();
        }
    }
    buf
}

fn render_not_ready(buf: &mut String, access: &TicketingAccessState, note: &str) {
    writeln!(
        buf,
        "{}",
        format!("🎟️  Tickets — {}", access.provider_label).bold()
    )
    .unwrap();
    writeln!(buf, "{}", note.yellow()).unwrap();
}

fn render_ready(buf: &mut String, r: &TicketStatsReady) {
    writeln!(
        buf,
        "{}",
        format!("🎟️  Tickets — {}", r.provider_label).bold()
    )
    .unwrap();
    writeln!(
        buf,
        "{}",
        format!(
            "   Amounts {} VAT{}",
            if r.amounts_include_vat {
                "include"
            } else {
                "exclude"
            },
            if r.revenue_apportioned {
                " · per-type revenue is apportioned across mixed orders"
            } else {
                ""
            }
        )
        .dimmed()
    )
    .unwrap();

    writeln!(buf).unwrap();
    row(
        buf,
        "Tickets:",
        &r.ticket_counts.all.to_string(),
        "Paid:",
        &r.ticket_counts.paid.to_string(),
    );
    row(
        buf,
        "Free:",
        &r.ticket_counts.free.to_string(),
        "Capacity:",
        // 0 means never configured — see `lib/tickets/config`.
        &if r.capacity == 0 {
            "not set".to_string()
        } else {
            r.capacity.to_string()
        },
    );
    row(
        buf,
        "Participants:",
        &r.participants.participants.to_string(),
        "Seats used:",
        &r.seats_used.to_string(),
    );
    render_participants(buf, &r.participants);

    render_revenue(buf, r);
    render_progress(buf, &r.analysis.paid);
    render_categories(buf, r);
    render_free(buf, &r.free_ticket_allocation);
    render_sponsor_tiers(buf, r);
}

fn render_participants(buf: &mut String, p: &ParticipantTally) {
    let mut left = Vec::new();
    if p.add_ons_with_seat > 0 {
        left.push(format!(
            "{} held by an attendee",
            plural(p.add_ons_with_seat, "add-on")
        ));
    }
    if p.add_ons_without_seat > 0 {
        left.push(format!(
            "{} with no ticket",
            plural(p.add_ons_without_seat, "add-on")
        ));
    }
    if p.repeat_tickets > 0 {
        left.push(plural(p.repeat_tickets, "repeat email"));
    }
    if !left.is_empty() {
        writeln!(
            buf,
            "{}",
            format!("   {} not counted as participants", left.join(", ")).dimmed()
        )
        .unwrap();
    }

    // The basis, named rather than asserted — the same wording the admin page
    // uses. `proposed` is the WEAKER claim: the role is suggested, not applied.
    let basis = match p.role_basis {
        RoleBasis::Declared => return,
        RoleBasis::Proposed => "Assumes suggested ticket type roles — confirm them on Ticket Types",
        RoleBasis::Unknown => {
            "Assumes ticket types with no role seat someone — set roles on Ticket Types"
        }
    };
    writeln!(buf, "{}", format!("   ⓘ {basis}").yellow()).unwrap();
}

fn render_revenue(buf: &mut String, r: &TicketStatsReady) {
    writeln!(buf).unwrap();
    writeln!(buf, "{}", "💰 Revenue (paid tickets)".bold()).unwrap();
    row(
        buf,
        "Total Revenue:",
        &money(r.statistics.total_revenue),
        "Orders:",
        &r.statistics.total_orders.to_string(),
    );
    row(
        buf,
        "Average Price:",
        &money(r.statistics.average_ticket_price),
        "Paid Tickets:",
        &r.statistics.total_paid_tickets.to_string(),
    );
}

fn render_progress(buf: &mut String, outcome: &AnalysisOutcome) {
    writeln!(buf).unwrap();
    match outcome {
        AnalysisOutcome::Ok { analysis } => {
            let p = &analysis.performance;
            let emoji = if p.is_on_track { "✅" } else { "⚠️" };
            writeln!(buf, "{}", format!("{emoji} Target Progress").bold()).unwrap();
            row(
                buf,
                "Actual Progress:",
                &format!("{:.1}%", p.current_percentage),
                "Current Target:",
                &format!("{:.1}%", p.target_percentage),
            );
            let variance = if p.variance >= 0.0 {
                format!("+{:.1}% ahead", p.variance).green().to_string()
            } else {
                format!("{:.1}% behind", p.variance).red().to_string()
            };
            writeln!(buf, "  {:<24}{variance}", "Variance:").unwrap();
            if let Some(m) = &p.next_milestone {
                writeln!(
                    buf,
                    "  🎯 Next Milestone: {} in {} days ({})",
                    m.label, m.days_away, m.date
                )
                .unwrap();
            }
        }
        // A real zero, not a failure.
        AnalysisOutcome::Empty => {
            writeln!(buf, "{}", "📈 Target Progress".bold()).unwrap();
            writeln!(
                buf,
                "{}",
                "  No tickets sold yet — nothing to analyse.".dimmed()
            )
            .unwrap();
        }
        // NOT substituted with zeros: we do not know the numbers.
        AnalysisOutcome::Unavailable { error } => {
            writeln!(buf, "{}", "⚠️ Target Progress".bold()).unwrap();
            writeln!(
                buf,
                "{}",
                format!("  Sales analysis failed — progress unknown: {error}").yellow()
            )
            .unwrap();
        }
    }
}

fn render_categories(buf: &mut String, r: &TicketStatsReady) {
    if r.category_stats.is_empty() {
        return;
    }
    writeln!(buf).unwrap();
    writeln!(buf, "{}", "Breakdown by Paid Ticket Category:".bold()).unwrap();
    for c in &r.category_stats {
        writeln!(
            buf,
            "  {}: {} tickets ({:.1}%) · {} · {} orders",
            c.category,
            c.count,
            c.percentage,
            money(c.revenue),
            c.orders
        )
        .unwrap();
    }
}

fn render_free(buf: &mut String, a: &FreeTicketAllocation) {
    writeln!(buf).unwrap();
    writeln!(buf, "{}", "🎁 Complimentary Tickets".bold()).unwrap();
    for (label, cat) in [
        ("Sponsors", &a.sponsors),
        ("Speakers", &a.speakers),
        ("Organizers", &a.organizers),
    ] {
        free_row(buf, label, cat);
    }

    let covers = if a.claimed_covers.is_empty() {
        "no category".to_string()
    } else {
        a.claimed_covers.join(", ")
    };
    writeln!(
        buf,
        "  {:<14}{} / {} claimed{}",
        "Total:",
        a.total_claimed.label(),
        a.claimed_allocated.label(),
        // The claimed total is PARTIAL — never present it as the whole event.
        if a.claimed_covers.len() == 3 {
            String::new()
        } else {
            format!(" ({covers} only)")
        }
    )
    .unwrap();
    writeln!(
        buf,
        "{}",
        format!("   {} seats allocated in total", a.total_allocated.label()).dimmed()
    )
    .unwrap();
}

fn free_row(buf: &mut String, label: &str, cat: &FreeAllocationCategory) {
    // `?`, never `0`: an unclaimed count and an uncountable one are not the same.
    let claimed = cat.claimed.label();
    let claimed = if cat.claimed.is_unknown() {
        claimed.yellow().to_string()
    } else {
        claimed
    };
    writeln!(
        buf,
        "  {:<14}{claimed} / {} claimed",
        format!("{label}:"),
        cat.allocated.label()
    )
    .unwrap();
    let source = if cat.from_provider {
        " (provider's own counter)"
    } else {
        ""
    };
    writeln!(buf, "{}", format!("   {}{source}", cat.status).dimmed()).unwrap();
}

fn render_sponsor_tiers(buf: &mut String, r: &TicketStatsReady) {
    if r.sponsor_tickets_by_tier.is_empty() {
        return;
    }
    writeln!(buf).unwrap();
    writeln!(
        buf,
        "{}",
        format!(
            "Sponsor Seats by Tier ({} allocated):",
            r.sponsor_allocation_total
        )
        .bold()
    )
    .unwrap();
    let mut tiers: Vec<_> = r.sponsor_tickets_by_tier.iter().collect();
    tiers.sort_by_key(|(_, data)| std::cmp::Reverse(data.tickets));
    for (tier, data) in tiers {
        writeln!(
            buf,
            "  {tier}: {} seats · {} sponsors · {} per sponsor",
            data.tickets, data.sponsors, data.tickets_per_sponsor
        )
        .unwrap();
    }
}

/// Whole units only, the way `display::status` prints money. The payload
/// carries no currency, so neither does this.
#[allow(clippy::cast_possible_truncation)]
fn money(amount: f64) -> String {
    format_thousands(amount as i64)
}

fn plural(n: u64, word: &str) -> String {
    format!("{n} {word}{}", if n == 1 { "" } else { "s" })
}

fn row(buf: &mut String, l1: &str, v1: &str, l2: &str, v2: &str) {
    writeln!(buf, "  {l1:<24}{v1:<10}  {l2:<24}{v2}").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ready_json;

    fn render(value: &serde_json::Value) -> String {
        let stats: TicketStats = serde_json::from_value(value.clone()).unwrap();
        render_ticket_stats(&stats)
    }

    /// The whole point: an uncountable claim must never read as `0`.
    #[test]
    fn unknown_claim_never_renders_as_zero() {
        let out = render(&ready_json("declared"));
        let organizers = out
            .lines()
            .find(|l| l.contains("Organizers:"))
            .expect("organizer row");

        assert!(organizers.contains('?'), "got: {organizers}");
        assert!(!organizers.contains("0 / 9"), "got: {organizers}");
        assert!(out.contains("cannot be told apart"));
        // The partial total names what it covers.
        assert!(out.contains("(sponsors, speakers only)"));
    }

    #[test]
    fn proposed_basis_is_stated() {
        let out = render(&ready_json("proposed"));
        assert!(out.contains("Assumes suggested ticket type roles"));

        let unknown = render(&ready_json("unknown"));
        assert!(unknown.contains("Assumes ticket types with no role"));

        let declared = render(&ready_json("declared"));
        assert!(!declared.contains("Assumes"));
    }

    #[test]
    fn vat_and_apportioning_are_stated() {
        let out = render(&ready_json("declared"));
        assert!(out.contains("Amounts exclude VAT"));
        assert!(out.contains("apportioned"));
    }

    /// A failed read is a failure on screen, not an empty conference.
    #[test]
    fn error_state_renders_as_failure() {
        let out = render(&serde_json::json!({
            "state": "error",
            "message": "Unable to fetch tickets: 502"
        }));
        assert!(out.contains("unavailable"));
        assert!(out.contains("502"));
        assert!(!out.contains("0 / 0"));
    }

    #[test]
    fn non_ready_states_name_the_provider() {
        for (state, needle) in [
            ("unconfigured", "not configured"),
            ("unavailable", "unavailable"),
            ("disabled", "disabled"),
        ] {
            let out = render(&serde_json::json!({
                "state": state,
                "providerType": "tito",
                "providerLabel": "Tito"
            }));
            assert!(out.contains("Tito"), "{state}: {out}");
            assert!(out.contains(needle), "{state}: {out}");
        }
    }

    #[test]
    fn failed_analysis_is_not_zero_progress() {
        let mut value = ready_json("declared");
        value["analysis"]["paid"] =
            serde_json::json!({ "status": "unavailable", "error": "processor blew up" });
        let out = render(&value);

        assert!(out.contains("progress unknown"));
        // No stand-in progress figure at all — not even a zeroed one.
        assert!(!out.contains("Actual Progress"));
        assert!(!out.contains("Variance"));
    }

    #[test]
    fn empty_analysis_says_no_sales_yet() {
        let mut value = ready_json("declared");
        value["analysis"]["paid"] = serde_json::json!({ "status": "empty" });
        let out = render(&value);

        assert!(out.contains("No tickets sold yet"));
        assert!(!out.contains("progress unknown"));
    }
}
