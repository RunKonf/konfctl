use serde::{Deserialize, Serialize};

use super::null_to_vec;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConferenceStatusSummary {
    pub conference_title: String,
    pub last_updated: String,
    pub sponsors: Option<SponsorPipeline>,
    pub proposals: Option<ProposalSummary>,
    pub tickets: Option<TicketSummary>,
    pub target_progress: Option<TargetProgress>,
    #[serde(default, deserialize_with = "null_to_vec")]
    pub errors: Vec<SectionError>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SponsorPipeline {
    pub total_sponsors: usize,
    pub active_deals: usize,
    pub closed_won_count: usize,
    pub closed_lost_count: usize,
    pub total_contract_value: f64,
    pub contract_currency: String,
    #[serde(default)]
    pub by_status: std::collections::HashMap<String, usize>,
    #[serde(default)]
    pub by_contract_status: std::collections::HashMap<String, usize>,
    #[serde(default)]
    pub by_invoice_status: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalSummary {
    pub total: usize,
    pub submitted: usize,
    pub accepted: usize,
    pub confirmed: usize,
    pub rejected: usize,
    pub withdrawn: usize,
    #[serde(default)]
    pub by_status: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketSummary {
    pub paid_tickets: usize,
    pub total_revenue: f64,
    pub total_tickets: usize,
    pub sponsor_tickets: usize,
    pub speaker_tickets: usize,
    pub organizer_tickets: usize,
    pub free_tickets_claimed: usize,
    pub free_ticket_claim_rate: f64,
    #[serde(default)]
    pub category_breakdown: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetProgress {
    pub current_percentage: f64,
    pub target_percentage: f64,
    pub variance: f64,
    pub is_on_track: bool,
    pub capacity: usize,
    pub next_milestone: Option<Milestone>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Milestone {
    pub label: String,
    pub days_away: i64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionError {
    pub section: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_full_summary() {
        let json = serde_json::json!({
            "conferenceTitle": "Cloud Native Days Norway 2026",
            "lastUpdated": "2026-03-30T10:00:00Z",
            "sponsors": {
                "totalSponsors": 42,
                "activeDeals": 37,
                "closedWonCount": 5,
                "closedLostCount": 0,
                "totalContractValue": 180_000.0,
                "contractCurrency": "NOK",
                "byStatus": {"closed-won": 5, "contacted": 26, "negotiating": 4, "prospect": 7},
                "byContractStatus": {"contract-signed": 3, "none": 36},
                "byInvoiceStatus": {"not-sent": 42},
                "byStatusValue": {}
            },
            "proposals": {
                "total": 44,
                "submitted": 44,
                "accepted": 0,
                "confirmed": 0,
                "rejected": 0,
                "withdrawn": 0,
                "byStatus": {"submitted": 44}
            },
            "tickets": {
                "paidTickets": 5,
                "totalRevenue": 12500.0,
                "totalTickets": 5,
                "sponsorTickets": 0,
                "speakerTickets": 5,
                "organizerTickets": 6,
                "freeTicketsClaimed": 0,
                "freeTicketClaimRate": 0.0,
                "categoryBreakdown": {
                    "Early Bird: Conference Only (1 day)": 4,
                    "Early Bird: Workshop + Conference (2 days)": 1
                }
            },
            "targetProgress": {
                "currentPercentage": 1.3,
                "targetPercentage": 0.0,
                "variance": 1.3,
                "isOnTrack": true,
                "capacity": 400,
                "nextMilestone": {"label": "Early Bird Close", "daysAway": 30}
            },
            "errors": []
        });

        let summary: ConferenceStatusSummary = serde_json::from_value(json).unwrap();
        assert_eq!(summary.conference_title, "Cloud Native Days Norway 2026");
        assert_eq!(summary.sponsors.as_ref().unwrap().total_sponsors, 42);
        assert_eq!(summary.proposals.as_ref().unwrap().total, 44);
        assert_eq!(summary.tickets.as_ref().unwrap().paid_tickets, 5);
        assert_eq!(summary.target_progress.as_ref().unwrap().capacity, 400);
        assert!(summary.errors.is_empty());
    }

    #[test]
    fn deserialize_minimal_summary() {
        let json = serde_json::json!({
            "conferenceTitle": "Test Conference",
            "lastUpdated": "2026-01-01T00:00:00Z",
            "sponsors": null,
            "proposals": null,
            "tickets": null,
            "targetProgress": null,
            "errors": [{"section": "tickets", "message": "not configured"}]
        });

        let summary: ConferenceStatusSummary = serde_json::from_value(json).unwrap();
        assert!(summary.sponsors.is_none());
        assert!(summary.proposals.is_none());
        assert!(summary.tickets.is_none());
        assert!(summary.target_progress.is_none());
        assert_eq!(summary.errors.len(), 1);
        assert_eq!(summary.errors[0].section, "tickets");
    }
}
