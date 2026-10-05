use crate::types::ScheduleStatus;
use clap::Args;

#[derive(Args, Clone)]
pub struct ListArgs {
    /// Filter by status
    #[arg(long, value_enum)]
    pub status: Option<ScheduleStatus>,
}

#[derive(Args, Clone)]
pub struct AddTalkArgs {
    /// Schedule ID
    pub id: String,

    /// Track index (0-indexed)
    #[arg(long)]
    pub track: usize,

    /// Proposal ID
    #[arg(long, required_unless_present("placeholder"))]
    pub proposal: Option<String>,

    /// Placeholder title (e.g. "Lunch Break")
    #[arg(long, required_unless_present("proposal"))]
    pub placeholder: Option<String>,

    /// Start time (HH:mm)
    #[arg(long)]
    pub start: String,

    /// End time (HH:mm)
    #[arg(long)]
    pub end: String,
}

#[derive(Args, Clone)]
pub struct RemoveTalkArgs {
    /// Schedule ID
    pub id: String,

    /// Track index (0-indexed)
    #[arg(long)]
    pub track: usize,

    /// Talk index (0-indexed)
    #[arg(long)]
    pub talk: usize,
}
