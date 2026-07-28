use std::fmt::Write;

use colored::Colorize;

use crate::types::{Proposal, ProposalStatus, portable_text_to_plain};

/// Render proposal details into a `String` (for scrollable views, etc.).
pub fn render_proposal_detail(proposal: &Proposal) -> String {
    let mut buf = String::new();
    writeln!(buf, "{}", proposal.title.bold()).unwrap();
    writeln!(buf, "ID:       {}", proposal.id).unwrap();
    writeln!(buf, "Status:   {}", colorize_status(proposal.status)).unwrap();
    if let Some(format) = proposal.format {
        writeln!(buf, "Format:   {format}").unwrap();
    }
    if let Some(level) = proposal.level.as_str() {
        writeln!(buf, "Level:    {}", capitalize(level)).unwrap();
    }
    if let Some(language) = proposal.language.as_str() {
        writeln!(buf, "Language: {}", capitalize(language)).unwrap();
    }

    if !proposal.speakers.is_empty() {
        writeln!(buf, "\nSpeakers:").unwrap();
        for s in &proposal.speakers {
            let email = s.email.as_str().unwrap_or("");
            let mut line = format!("  - {} <{}>", s.name, email);
            if !s.flags.is_empty() {
                let flags: Vec<String> = s.flags.iter().map(|f| format!("[{f}]")).collect();
                write!(line, " {}", flags.join(" ").yellow()).unwrap();
            }
            writeln!(buf, "{line}").unwrap();
        }
    }

    if !proposal.topics.is_empty() {
        let topics: Vec<&str> = proposal.topics.iter().filter_map(|t| t.title()).collect();
        if !topics.is_empty() {
            writeln!(buf, "\nTopics: {}", topics.join(", ")).unwrap();
        }
    }

    if !proposal.description.is_empty() {
        let desc = portable_text_to_plain(&proposal.description);
        if !desc.is_empty() {
            writeln!(buf, "\nDescription:\n{desc}").unwrap();
        }
    }

    if let Some(outline) = proposal.outline.as_str()
        && !outline.is_empty()
    {
        writeln!(buf, "\nOutline:\n{outline}").unwrap();
    }

    if !proposal.reviews.is_empty() {
        writeln!(buf, "\nReviews ({}):", proposal.reviews.len()).unwrap();
        for r in &proposal.reviews {
            let reviewer = r
                .reviewer
                .as_ref()
                .map_or("Anonymous", |rev| rev.name.as_str());
            if let Some(score) = &r.score {
                writeln!(
                    buf,
                    "  {} — Content: {} Relevance: {} Speaker: {} (total: {:.0})",
                    reviewer,
                    score.content,
                    score.relevance,
                    score.speaker,
                    score.total()
                )
                .unwrap();
            } else {
                writeln!(buf, "  {reviewer} — (no score)").unwrap();
            }
            if !r.comment.is_null() {
                if let Some(comment) = r.comment.as_str() {
                    if !comment.is_empty() {
                        writeln!(buf, "    {}", comment.dimmed()).unwrap();
                    }
                } else {
                    writeln!(buf, "    {}", r.comment.to_string().dimmed()).unwrap();
                }
            }
        }
    }

    buf
}

/// Print proposal details to stdout.
pub fn print_proposal_detail(proposal: &Proposal) {
    print!("{}", render_proposal_detail(proposal));
}

fn colorize_status(status: ProposalStatus) -> String {
    let label = status.to_string();
    match status {
        ProposalStatus::Submitted => label.yellow().to_string(),
        ProposalStatus::Accepted => label.green().to_string(),
        ProposalStatus::Confirmed => label.green().bold().to_string(),
        ProposalStatus::Rejected => label.red().to_string(),
        ProposalStatus::Waitlisted => label.cyan().to_string(),
        ProposalStatus::Withdrawn
        | ProposalStatus::Draft
        | ProposalStatus::Deleted
        | ProposalStatus::Unknown => label.dimmed().to_string(),
    }
}

pub fn pad_and_colorize_status(status: ProposalStatus, width: usize) -> String {
    let label = status.to_string();
    let padded = format!("{label:<width$}");
    match status {
        ProposalStatus::Submitted => padded.yellow().to_string(),
        ProposalStatus::Accepted => padded.green().to_string(),
        ProposalStatus::Confirmed => padded.green().bold().to_string(),
        ProposalStatus::Rejected => padded.red().to_string(),
        ProposalStatus::Waitlisted => padded.cyan().to_string(),
        ProposalStatus::Withdrawn
        | ProposalStatus::Draft
        | ProposalStatus::Deleted
        | ProposalStatus::Unknown => padded.dimmed().to_string(),
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}
