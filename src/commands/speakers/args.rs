use clap::{Args, Subcommand};
use serde::Serialize;

use crate::types::{ProposalSortBy, ProposalStatus, SortOrder, SpeakerFlag};

#[derive(Args)]
pub struct SpeakerArgs {
    #[command(subcommand)]
    pub command: SpeakerCommand,
}

#[derive(Subcommand)]
pub enum SpeakerCommand {
    /// List speakers (defaults to accepted/confirmed speakers)
    List(ListArgs),
    /// Show full speaker details
    Get {
        /// Speaker ID
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Add a new speaker profile
    Add(CreateArgs),
    /// Delete a speaker
    Delete {
        /// Speaker ID
        id: String,

        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Send a broadcast email to ALL speakers (use with caution!)
    Broadcast {
        /// Email subject
        #[arg(long, requires = "message")]
        subject: Option<String>,

        /// Email message (plain text)
        #[arg(long, requires = "subject")]
        message: Option<String>,

        /// Sync with newsletter audience before sending (or only sync if subject/message are omitted)
        #[arg(long)]
        #[arg(required_unless_present_any = ["subject", "message"])]
        sync: bool,
    },
    /// Find a speaker by email or create a new one if not found
    FindOrCreate(FindOrCreateArgs),
}

#[derive(Args, Serialize)]
pub struct FindOrCreateArgs {
    /// Speaker email
    pub email: String,

    /// Speaker name (required for creation)
    pub name: String,

    /// Job title
    #[arg(long)]
    pub title: Option<String>,

    /// Company name
    #[arg(long)]
    pub company: Option<String>,
}

#[derive(Args, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListArgs {
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

    /// Search across names and emails
    #[arg(long = "search")]
    #[serde(rename = "query", skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,

    /// Search across ALL speakers in the database (not just this conference)
    #[arg(long)]
    #[serde(skip)]
    pub all: bool,

    /// Filter by proposal status (comma-separated, defaults to accepted,confirmed)
    #[arg(long, value_delimiter = ',', value_enum)]
    #[serde(skip)]
    pub status: Option<Vec<ProposalStatus>>,

    /// Sort by field
    #[arg(long = "sort", value_enum, default_value_t = ProposalSortBy::Speaker)]
    #[serde(skip)]
    pub sort: ProposalSortBy,

    /// Sort order
    #[arg(long = "order", value_enum, default_value_t = SortOrder::Asc)]
    #[serde(skip)]
    pub order: SortOrder,
}

impl ListArgs {
    pub fn has_cli_filters(&self) -> bool {
        self.query.is_some()
            || self.status.is_some()
            || self.sort != ProposalSortBy::Speaker
            || self.order != SortOrder::Asc
    }
}

#[derive(Args, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateArgs {
    /// Optional specific ID (UUID)
    #[arg(long)]
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Speaker name
    pub name: String,

    /// Speaker email
    pub email: String,

    /// Job title
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Company name
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,

    /// Biography (plain text)
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,

    /// Image URL
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    /// Social links (comma-separated)
    #[arg(long, value_delimiter = ',')]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<String>>,

    /// Speaker flags (comma-separated)
    #[arg(long, value_delimiter = ',', value_enum)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<SpeakerFlag>>,
}
