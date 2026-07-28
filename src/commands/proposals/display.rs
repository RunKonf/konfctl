use crate::types::{Proposal, ProposalFormat, ProposalSortBy, ProposalStatus, SortOrder};
use crate::{display, ui};

pub const TABLE_HEADER: &str = "STATUS       FORMAT           TITLE · SPEAKER";

#[derive(Debug, Clone, Default)]
pub struct Filters {
    pub search: Option<String>,
    pub statuses: Vec<ProposalStatus>,
    pub formats: Vec<ProposalFormat>,
    pub sort_by: ProposalSortBy,
    pub sort_order: SortOrder,
}

pub fn format_item(p: &Proposal) -> String {
    let speakers: Vec<&str> = p.speakers.iter().map(|s| s.name.as_str()).collect();
    let speaker_str = speakers.join(", ");
    let format = p.format.map_or("-".to_string(), |f| f.label().to_string());
    let status = display::pad_and_colorize_status(p.status, 12);

    let prefix_len = 12 + 1 + 16 + 1;
    let prefix = format!("{status} {format:<16} ");
    let remaining = ui::term_width().saturating_sub(prefix_len + 4);
    let title_budget = remaining * 2 / 3;
    let speaker_budget = remaining.saturating_sub(title_budget + 3);

    let title = ui::truncate(&p.title, title_budget);
    if speaker_str.is_empty() {
        format!("{prefix}{title}")
    } else {
        let speaker = ui::truncate(&speaker_str, speaker_budget);
        format!("{prefix}{title} · {speaker}")
    }
}

pub fn filter_summary(filters: &Filters) -> String {
    let status_part: String = if filters.statuses.is_empty() {
        "all statuses".into()
    } else {
        filters
            .statuses
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };
    let format_part = if filters.formats.is_empty() {
        "all formats".into()
    } else {
        filters
            .formats
            .iter()
            .map(|f: &ProposalFormat| f.label())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let dir = if filters.sort_order == SortOrder::Asc {
        "↑"
    } else {
        "↓"
    };
    format!(
        "status: {status_part} | format: {format_part} | sort: {:?}{dir}",
        filters.sort_by
    )
}
