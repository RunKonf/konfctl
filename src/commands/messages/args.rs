use clap::{Args, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Args)]
pub struct MessageArgs {
    #[command(subcommand)]
    pub command: MessageCommand,
}

#[derive(Subcommand)]
pub enum MessageCommand {
    /// List conversations in the inbox
    List(ListArgs),

    /// Get a conversation and its messages
    Get {
        /// Conversation ID
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Reply to an existing conversation
    Reply {
        /// Conversation ID
        id: String,

        /// Message body
        #[arg(long, short = 'm')]
        message: String,
    },

    /// Start a new conversation with a speaker
    New {
        /// Speaker ID
        #[arg(long)]
        speaker: String,

        /// Subject line
        #[arg(long)]
        subject: String,

        /// Message body
        #[arg(long, short = 'm')]
        message: String,
    },

    /// Change the status of a conversation (open/resolved)
    Status {
        /// Conversation ID
        id: String,

        /// New status
        #[arg(value_enum)]
        status: ConversationStatusEnum,
    },

    /// Assign a conversation to an organizer (omit --to to unassign)
    Assign {
        /// Conversation ID
        id: String,

        /// Organizer/Speaker ID to assign to
        #[arg(long)]
        to: Option<String>,
    },

    /// Toggle the global archive state of a conversation
    Archive {
        /// Conversation ID
        id: String,

        /// Set to unarchive
        #[arg(long)]
        unarchive: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum InboxView {
    #[default]
    Active,
    NeedsReply,
    MyTeams,
    Unassigned,
    Mine,
    Resolved,
    Archived,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConversationStatusEnum {
    Open,
    Resolved,
}

#[derive(Args, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListArgs {
    /// Inbox view to use
    #[arg(long, value_enum, default_value = "active")]
    pub view: InboxView,

    /// Pagination cursor
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

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
