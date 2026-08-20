use clap::Args;
use serde::Serialize;

use crate::types::{SortOrder, SponsorSortBy, SponsorStatus, SponsorView};

#[allow(clippy::struct_excessive_bools)]
#[derive(Args, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListArgs {
    /// CRM View to use
    #[arg(long, value_enum)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view: Option<SponsorView>,

    /// Search across sponsor names, websites, and contact details
    #[arg(long = "search")]
    #[serde(rename = "searchQuery", skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,

    /// Filter by status (comma-separated)
    #[arg(long, value_delimiter = ',', value_enum)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Vec<SponsorStatus>>,

    /// Filter by my assigned items
    #[arg(long = "mine")]
    #[serde(rename = "myAssignedOnly", skip_serializing_if = "std::ops::Not::not")]
    pub mine: bool,

    /// Filter by organizer (speaker ID)
    #[arg(long = "assigned-to")]
    #[serde(rename = "assignedTo", skip_serializing_if = "Option::is_none")]
    pub assigned_to: Option<String>,

    /// Find leads without an assignee
    #[arg(long = "unassigned")]
    #[serde(rename = "unassignedOnly", skip_serializing_if = "std::ops::Not::not")]
    pub unassigned: bool,

    /// Filter by tags (comma-separated)
    #[arg(long, value_delimiter = ',')]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Filter by tier IDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tiers: Option<Vec<String>>,

    /// Sort by field
    #[arg(long = "sort", value_enum)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<SponsorSortBy>,

    /// Sort order
    #[arg(long = "order", value_enum)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<SortOrder>,

    /// Filter by staleness (inactive for N+ days)
    #[arg(long = "stale")]
    #[serde(rename = "staleDays", skip_serializing_if = "Option::is_none")]
    pub stale_days: Option<u32>,

    /// Filter by follow-up due (nextFollowUpAt <= today)
    #[arg(long = "due")]
    #[serde(rename = "followUpDue", skip_serializing_if = "std::ops::Not::not")]
    pub due: bool,

    /// Filter by having any scheduled follow-up
    #[arg(long = "has-follow-up")]
    #[serde(rename = "hasFollowUp", skip_serializing_if = "std::ops::Not::not")]
    pub has_follow_up: bool,

    /// Filter by having a contact email
    #[arg(long = "has-contact")]
    #[serde(rename = "hasContactInfo", skip_serializing_if = "std::ops::Not::not")]
    pub has_contact: bool,

    /// Output minimal JSON fields
    #[arg(long)]
    #[serde(skip)]
    pub compact: bool,

    /// Limit the number of items returned
    #[arg(long)]
    #[serde(skip)]
    pub limit: Option<usize>,

    /// Output as JSON
    #[arg(long)]
    #[serde(skip)]
    pub json: bool,
}

#[derive(Args, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArgs {
    /// Sponsor-for-conference ID
    #[serde(skip)]
    pub id: String,

    /// Update next follow-up date (ISO 8601, e.g. 2026-06-15)
    #[arg(long)]
    #[serde(rename = "nextFollowUpAt", skip_serializing_if = "Option::is_none")]
    pub next_follow_up: Option<String>,

    /// Update `LinkedIn` URL for the organization
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linkedin_url: Option<String>,

    /// Internal notes
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    /// Assign an organizer (speaker ID)
    #[arg(long = "assigned-to")]
    #[serde(rename = "assignedTo", skip_serializing_if = "Option::is_none")]
    pub assigned_to: Option<String>,

    /// Update sponsorship tier ID
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,

    /// Update contract value
    #[arg(long)]
    #[serde(rename = "contractValue", skip_serializing_if = "Option::is_none")]
    pub contract_value: Option<f64>,

    /// Update contract currency (e.g. NOK, USD)
    #[arg(long)]
    #[serde(rename = "contractCurrency", skip_serializing_if = "Option::is_none")]
    pub contract_currency: Option<String>,
}

#[derive(Args)]
pub struct UpdateContactsArgs {
    /// Sponsor-for-conference ID
    pub id: String,

    /// New primary contact name
    #[arg(long)]
    pub name: String,

    /// New primary contact email
    #[arg(long)]
    pub email: String,
}

#[derive(Args)]
pub struct EmailArgs {
    /// Sponsor-for-conference ID
    pub id: String,

    /// Template slug to use (interactive picker if omitted)
    #[arg(long)]
    pub template: Option<String>,

    /// Override the email subject
    #[arg(long)]
    pub subject: Option<String>,

    /// Use this message body directly (skip template selection)
    #[arg(long)]
    pub message: Option<String>,

    /// Open $EDITOR to edit the message before sending
    #[arg(long)]
    pub edit: bool,

    /// Preview the email without sending
    #[arg(long)]
    pub dry_run: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct CreateArgs {
    /// Sponsor organization name
    pub name: String,

    /// Website URL
    #[arg(long)]
    pub website: Option<String>,

    /// Primary contact name
    #[arg(long)]
    pub contact_name: Option<String>,

    /// Primary contact email
    #[arg(long)]
    pub contact_email: Option<String>,

    /// Initial status
    #[arg(long, default_value = "prospect", value_enum)]
    pub status: SponsorStatus,

    /// Internal notes
    #[arg(long)]
    pub notes: Option<String>,
}

#[derive(Args)]
pub struct NoteArgs {
    /// Sponsor-for-conference ID
    pub id: String,

    /// Type of activity
    #[arg(long, default_value = "note", value_enum)]
    pub kind: crate::types::ActivityType,

    /// Description of the activity
    pub description: String,
}
