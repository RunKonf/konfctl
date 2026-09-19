//! The payload of `tickets.admin.summary`, mirrored field for field.
//!
//! The server computes every figure here (`@/lib/tickets/summary` in the
//! website repo) and deliberately refuses to flatten what it does not know.
//! These types refuse the same things: a claim that could not be counted stays
//! `Count::Unknown` instead of becoming `0`, a headcount resting on proposed
//! ticket-type roles carries its `RoleBasis`, and a failed provider read is a
//! different variant from an event with no tickets yet.
//!
//! Nothing in this module derives a number. If a surface needs one, it belongs
//! in the procedure.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// `'unknown'`, and only that string.
///
/// An externally tagged enum of one unit variant is exactly a string literal
/// on the wire, so a stray `"n/a"` fails to deserialize rather than passing as
/// an admission of ignorance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum UnknownTag {
    #[serde(rename = "unknown")]
    Unknown,
}

/// A count we have, or the admission that we do not have one (`number | 'unknown'`).
///
/// Never collapse this to a number: `0` claimed reads as "nobody claimed", and
/// the organizer row is uncountable by construction on every tenant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Count {
    Known(u64),
    Unknown(UnknownTag),
}

impl Count {
    pub fn is_unknown(self) -> bool {
        matches!(self, Self::Unknown(_))
    }

    /// What a human surface prints. `?` — never `0`.
    pub fn label(self) -> String {
        match self {
            Self::Known(n) => n.to_string(),
            Self::Unknown(_) => "?".to_string(),
        }
    }
}

/// The weakest basis any ticket's `admits` rested on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RoleBasis {
    /// Every ticket type's role was declared by a human.
    Declared,
    /// At least one role is a suggestion that has NOT been applied.
    Proposed,
    /// At least one type has neither a declaration nor a proposal.
    Unknown,
}

/// The four states `buildTicketSummary` can answer with.
///
/// `Error` is a provider read that FAILED; an event with no tickets is a
/// `Ready` summary whose analyses are `empty`. Collapsing the two would present
/// a failure as a confident zero.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum TicketStats {
    Ready(Box<TicketStatsReady>),
    Unconfigured(TicketingAccessState),
    Unavailable(TicketingAccessState),
    Disabled(TicketingAccessState),
    Error { message: String },
}

/// Nothing to compute, and which provider it would have come from.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketingAccessState {
    pub provider_type: String,
    pub provider_label: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketStatsReady {
    pub provider_type: String,
    pub provider_label: String,
    /// Whether the amounts below INCLUDE VAT, read off the adapter.
    pub amounts_include_vat: bool,
    /// Whether per-type revenue is an even split of per-order amounts.
    pub revenue_apportioned: bool,
    /// Sellable seats, or 0 when never configured — not a default.
    pub capacity: u64,
    pub ticket_counts: TicketCounts,
    pub participants: ParticipantTally,
    /// Chairs in the room: participants plus their repeat seats.
    pub seats_used: u64,
    pub analysis: Analyses,
    pub statistics: TicketStatistics,
    #[serde(default)]
    pub category_stats: Vec<CategoryStat>,
    pub free_ticket_allocation: FreeTicketAllocation,
    #[serde(default)]
    pub sponsor_tickets_by_tier: HashMap<String, SponsorTicketData>,
    /// Sponsor seats ALLOCATED, not redeemed.
    pub sponsor_allocation_total: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TicketCounts {
    pub all: u64,
    /// Bought, by GRANT rather than by price.
    pub paid: u64,
    /// Granted. The complement of `paid` over the same rule.
    pub free: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantTally {
    pub participants: u64,
    pub add_ons_with_seat: u64,
    pub add_ons_without_seat: u64,
    pub repeat_tickets: u64,
    pub role_basis: RoleBasis,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Analyses {
    /// Over the paid tickets — what the cards read.
    pub paid: AnalysisOutcome,
    /// Over everything — what the free-ticket toggle selects.
    pub all: AnalysisOutcome,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum AnalysisOutcome {
    Ok {
        analysis: TicketAnalysisResult,
    },
    /// No tickets to analyse. A real zero, not a failure.
    Empty,
    /// The analysis threw. We do NOT know the numbers.
    Unavailable {
        error: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketAnalysisResult {
    pub statistics: TicketStatistics,
    /// Chart data. Carried verbatim so `--json` loses nothing; never rendered.
    #[serde(default)]
    pub progression: serde_json::Value,
    pub performance: PerformanceMetrics,
    pub capacity: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceMetrics {
    pub current_percentage: f64,
    pub target_percentage: f64,
    pub variance: f64,
    pub is_on_track: bool,
    pub next_milestone: Option<NextMilestone>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NextMilestone {
    pub date: String,
    pub label: String,
    pub days_away: i64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketStatistics {
    pub total_paid_tickets: u64,
    pub total_revenue: f64,
    pub total_orders: u64,
    pub average_ticket_price: f64,
    #[serde(default)]
    pub category_breakdown: HashMap<String, u64>,
    pub sponsor_tickets: u64,
    pub speaker_tickets: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CategoryStat {
    pub category: String,
    pub count: u64,
    pub orders: u64,
    pub revenue: f64,
    /// Server-computed share of paid tickets. Do not recompute it here.
    pub percentage: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SponsorTicketData {
    pub sponsors: u64,
    pub tickets: u64,
    pub tickets_per_sponsor: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeTicketAllocation {
    pub sponsors: FreeAllocationCategory,
    pub speakers: FreeAllocationCategory,
    pub organizers: FreeAllocationCategory,
    pub total_allocated: Count,
    /// Claims over the categories `claimed_covers` names — a PARTIAL total.
    pub total_claimed: Count,
    /// The allowance of exactly those categories, so a rate divides like by like.
    pub claimed_allocated: Count,
    #[serde(default)]
    pub claimed_covers: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeAllocationCategory {
    pub allocated: Count,
    /// Seats taken. NOT clamped to `allocated`, and never zero-filled.
    pub claimed: Count,
    /// The count came from the provider's own redemption counter.
    pub from_provider: bool,
    /// One line saying where the number came from, or why there is none.
    pub status: String,
}

/// The `ready` fixture, shared with the renderer's tests.
/// The `ready` fixture, shared with the renderer's tests.
#[cfg(test)]
pub(crate) use tests::ready_json;

#[cfg(test)]
mod tests {
    use super::*;

    /// A `ready` payload with an unknown organizer claim and a declared basis.
    pub(crate) fn ready_json(role_basis: &str) -> serde_json::Value {
        serde_json::json!({
            "state": "ready",
            "providerType": "checkin",
            "providerLabel": "Checkin",
            "amountsIncludeVat": false,
            "revenueApportioned": true,
            "capacity": 400,
            "ticketCounts": { "all": 120, "paid": 96, "free": 24 },
            "participants": {
                "participants": 111,
                "addOnsWithSeat": 6,
                "addOnsWithoutSeat": 1,
                "repeatTickets": 2,
                "roleBasis": role_basis
            },
            "seatsUsed": 113,
            "analysis": {
                "paid": {
                    "status": "ok",
                    "analysis": {
                        "statistics": {
                            "totalPaidTickets": 96,
                            "totalRevenue": 240_000.0,
                            "totalOrders": 80,
                            "averageTicketPrice": 2500.0,
                            "categoryBreakdown": { "Early Bird": 96 },
                            "sponsorTickets": 0,
                            "speakerTickets": 0
                        },
                        "progression": [{ "date": "2026-01-01", "actualTickets": 4 }],
                        "performance": {
                            "currentPercentage": 24.0,
                            "targetPercentage": 30.0,
                            "variance": -6.0,
                            "isOnTrack": false,
                            "nextMilestone": {
                                "date": "2026-04-01",
                                "label": "Early Bird Close",
                                "daysAway": 12
                            }
                        },
                        "capacity": 400
                    }
                },
                "all": { "status": "empty" }
            },
            "statistics": {
                "totalPaidTickets": 96,
                "totalRevenue": 240_000.0,
                "totalOrders": 80,
                "averageTicketPrice": 2500.0,
                "categoryBreakdown": { "Early Bird": 96 },
                "sponsorTickets": 0,
                "speakerTickets": 0
            },
            "categoryStats": [
                { "category": "Early Bird", "count": 96, "orders": 80, "revenue": 240_000.0, "percentage": 100.0 }
            ],
            "freeTicketAllocation": {
                "sponsors": {
                    "allocated": 40,
                    "claimed": 12,
                    "fromProvider": false,
                    "status": "Redemptions of 100%-off sponsor codes."
                },
                "speakers": {
                    "allocated": 30,
                    "claimed": 17,
                    "fromProvider": false,
                    "status": "13 invitations unclaimed · 0 speakers never invited."
                },
                "organizers": {
                    "allocated": 9,
                    "claimed": "unknown",
                    "fromProvider": false,
                    "status": "Organizer comps cannot be told apart from any other free ticket."
                },
                "totalAllocated": 79,
                "totalClaimed": 29,
                "claimedAllocated": 70,
                "claimedCovers": ["sponsors", "speakers"]
            },
            "sponsorTicketsByTier": {
                "Gold": { "sponsors": 4, "tickets": 24, "ticketsPerSponsor": 6 }
            },
            "sponsorAllocationTotal": 40
        })
    }

    fn ready(value: &serde_json::Value) -> TicketStatsReady {
        match serde_json::from_value::<TicketStats>(value.clone()).unwrap() {
            TicketStats::Ready(r) => *r,
            other => panic!("expected ready, got {other:?}"),
        }
    }

    #[test]
    fn deserializes_ready_payload() {
        let r = ready(&ready_json("declared"));

        assert_eq!(r.provider_label, "Checkin");
        assert!(!r.amounts_include_vat);
        assert!(r.revenue_apportioned);
        assert_eq!(r.ticket_counts.paid, 96);
        assert_eq!(r.seats_used, 113);
        assert_eq!(r.participants.role_basis, RoleBasis::Declared);
        assert_eq!(r.free_ticket_allocation.sponsors.claimed, Count::Known(12));
        assert!((r.category_stats[0].percentage - 100.0).abs() < f64::EPSILON);
        assert_eq!(r.sponsor_tickets_by_tier["Gold"].tickets_per_sponsor, 6);
        assert!(matches!(r.analysis.all, AnalysisOutcome::Empty));
        match r.analysis.paid {
            AnalysisOutcome::Ok { analysis } => {
                assert!(!analysis.performance.is_on_track);
                assert_eq!(analysis.performance.next_milestone.unwrap().days_away, 12);
                // Chart data survives the round trip for `--json`.
                assert!(analysis.progression.is_array());
            }
            other => panic!("expected ok, got {other:?}"),
        }
    }

    /// The one that must fail if `Count` is ever a plain number: the organizer
    /// claim is `'unknown'`, and it must survive as such.
    #[test]
    fn unknown_claim_is_not_a_number() {
        let r = ready(&ready_json("declared"));
        let organizers = &r.free_ticket_allocation.organizers;

        assert!(organizers.claimed.is_unknown());
        assert_ne!(organizers.claimed, Count::Known(0));
        assert_eq!(organizers.claimed.label(), "?");
        // And it goes back out as `'unknown'`, not as 0.
        assert_eq!(
            serde_json::to_value(organizers.claimed).unwrap(),
            serde_json::json!("unknown")
        );
    }

    #[test]
    fn unknown_is_the_only_accepted_string() {
        assert!(serde_json::from_value::<Count>(serde_json::json!("unknown")).is_ok());
        assert!(serde_json::from_value::<Count>(serde_json::json!("n/a")).is_err());
        assert!(serde_json::from_value::<Count>(serde_json::json!(null)).is_err());
    }

    #[test]
    fn deserializes_every_role_basis() {
        for (raw, expected) in [
            ("declared", RoleBasis::Declared),
            ("proposed", RoleBasis::Proposed),
            ("unknown", RoleBasis::Unknown),
        ] {
            let r = ready(&ready_json(raw));
            assert_eq!(r.participants.role_basis, expected);
        }
    }

    #[test]
    fn deserializes_non_ready_states() {
        for state in ["unconfigured", "unavailable", "disabled"] {
            let value = serde_json::json!({
                "state": state,
                "providerType": "tito",
                "providerLabel": "Tito"
            });
            let parsed: TicketStats = serde_json::from_value(value).unwrap();
            let ((TicketStats::Unconfigured(access), "unconfigured")
            | (TicketStats::Unavailable(access), "unavailable")
            | (TicketStats::Disabled(access), "disabled")) = (&parsed, state)
            else {
                panic!("state {state} deserialized as {parsed:?}")
            };
            assert_eq!(access.provider_label, "Tito");
        }
    }

    /// A FAILED provider read. Not an empty event, and it carries no provider.
    #[test]
    fn deserializes_error_state() {
        let parsed: TicketStats = serde_json::from_value(serde_json::json!({
            "state": "error",
            "message": "Unable to fetch tickets: 502"
        }))
        .unwrap();

        match parsed {
            TicketStats::Error { message } => assert!(message.contains("502")),
            other => panic!("expected error, got {other:?}"),
        }
    }

    #[test]
    fn empty_analysis_is_not_a_failed_one() {
        let empty: AnalysisOutcome =
            serde_json::from_value(serde_json::json!({ "status": "empty" })).unwrap();
        let failed: AnalysisOutcome =
            serde_json::from_value(serde_json::json!({ "status": "unavailable", "error": "boom" }))
                .unwrap();

        assert!(matches!(empty, AnalysisOutcome::Empty));
        assert!(matches!(failed, AnalysisOutcome::Unavailable { .. }));
    }
}
