use clap::{Args, ValueEnum};
use serde::Serialize;

use crate::types::{ProposalFormat, ProposalSortBy, ProposalStatus, ReviewStatus, SortOrder};

#[allow(clippy::struct_excessive_bools)]
#[derive(Args, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListArgs {
    /// Output as JSON (non-interactive)
    #[arg(long)]
    #[serde(skip)]
    pub json: bool,

    /// Output minimal JSON fields for agents (ID, Title, Status, Speakers)
    #[arg(long)]
    #[serde(skip)]
    pub compact: bool,

    /// Case-insensitive search across proposal titles and speaker names
    #[arg(long)]
    #[serde(rename = "searchQuery", skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,

    /// Filter by one or more statuses (comma-separated, e.g. submitted,accepted)
    #[arg(long, value_delimiter = ',', value_enum)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Vec<ProposalStatus>>,

    /// Filter by talk formats (comma-separated, e.g. `presentation_40,lightning_10`)
    #[arg(long, value_delimiter = ',', value_enum)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<Vec<ProposalFormat>>,

    /// Filter by technical level (comma-separated, e.g. beginner,expert)
    #[arg(long, value_delimiter = ',')]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<Vec<String>>,

    /// Filter by review status
    #[arg(long, value_enum)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_status: Option<ReviewStatus>,

    /// Show only unreviewed proposals (alias for --review-status unreviewed)
    #[arg(long)]
    #[serde(skip)]
    pub unreviewed: bool,

    /// Automatically hides multiple talks from the same speaker
    #[arg(long = "hide-multiple")]
    #[serde(rename = "hideMultipleTalks")]
    pub hide_multiple_talks: bool,

    /// Sort by field
    #[arg(long = "sort", value_enum, default_value_t = ProposalSortBy::Created)]
    pub sort_by: ProposalSortBy,

    /// Sort order
    #[arg(long = "order", value_enum, default_value_t = SortOrder::Desc)]
    pub sort_order: SortOrder,

    /// Limit the number of items returned
    #[arg(long)]
    #[serde(skip)]
    pub limit: Option<usize>,
}

#[derive(Args, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateArgs {
    /// Proposal title
    pub title: String,

    /// Talk format
    #[arg(long, value_enum)]
    pub format: Option<ProposalFormat>,

    /// Technical level
    #[arg(long)]
    pub level: Option<String>,

    /// Language
    #[arg(long)]
    pub language: Option<String>,

    /// Speaker IDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub speakers: Option<Vec<String>>,

    /// Target audiences (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub audiences: Option<Vec<String>>,

    /// Topic IDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub topics: Option<Vec<String>>,

    /// Accept Terms of Service
    #[arg(long)]
    pub tos: bool,

    /// Description/Abstract
    #[arg(long)]
    pub description: Option<String>,

    /// Outline
    #[arg(long)]
    pub outline: Option<String>,
}

#[derive(ValueEnum, Clone, Copy, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ProposalAction {
    Accept,
    Reject,
    Confirm,
    Withdraw,
}

#[derive(Args)]
pub struct ActionArgs {
    /// Proposal ID
    pub id: String,

    /// Action to perform
    #[arg(value_enum)]
    pub action: ProposalAction,

    /// Send notification email to speakers
    #[arg(long)]
    pub notify: bool,

    /// Internal/External comment regarding the action
    #[arg(long)]
    pub comment: Option<String>,
}

#[derive(Args, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArgs {
    /// Proposal ID
    #[serde(skip)]
    pub id: String,

    /// Proposal title
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Talk format
    #[arg(long, value_enum)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ProposalFormat>,

    /// Technical level
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,

    /// Language
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Speaker IDs (comma-separated, replaces current list)
    #[arg(long, value_delimiter = ',')]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speakers: Option<Vec<String>>,

    /// Description/Abstract
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Outline
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outline: Option<String>,

    /// Status
    #[arg(long, value_enum)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ProposalStatus>,
}

#[derive(Args)]
pub struct DeleteArgs {
    /// Proposal ID
    pub id: String,

    /// Skip confirmation prompt
    #[arg(long, short = 'y')]
    pub yes: bool,
}

#[derive(Args)]
pub struct ReviewArgs {
    /// Proposal ID
    pub id: String,

    /// Content score (1–5)
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=5),
          requires_all = ["relevance", "speaker", "comment"])]
    pub content: Option<u8>,

    /// Relevance score (1–5)
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=5),
          requires_all = ["content", "speaker", "comment"])]
    pub relevance: Option<u8>,

    /// Speaker score (1–5)
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=5),
          requires_all = ["content", "relevance", "comment"])]
    pub speaker: Option<u8>,

    /// Review comment
    #[arg(long, requires_all = ["content", "relevance", "speaker"])]
    pub comment: Option<String>,
}

impl ListArgs {
    pub fn has_cli_filters(&self) -> bool {
        self.search.is_some()
            || self.status.is_some()
            || self.format.is_some()
            || self.level.is_some()
            || self.review_status.is_some()
            || self.unreviewed
            || self.hide_multiple_talks
            || self.sort_by != ProposalSortBy::Created
            || self.sort_order != SortOrder::Desc
    }
}
