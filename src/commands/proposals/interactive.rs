use anyhow::Result;
use colored::Colorize;
use console::Key;
use dialoguer::FuzzySelect;

use crate::client::TrpcClient;
use crate::types::{Proposal, ProposalFormat, ProposalSortBy, ProposalStatus, SortOrder};
use crate::{config, display, ui};

use super::display::{Filters, TABLE_HEADER, filter_summary, format_item};
use super::review::prompt_and_submit_review;

const STATUSES: &[ProposalStatus] = &[
    ProposalStatus::Submitted,
    ProposalStatus::Accepted,
    ProposalStatus::Confirmed,
    ProposalStatus::Waitlisted,
    ProposalStatus::Rejected,
    ProposalStatus::Withdrawn,
];

const FORMATS: &[ProposalFormat] = &[
    ProposalFormat::Lightning10,
    ProposalFormat::Presentation20,
    ProposalFormat::Presentation25,
    ProposalFormat::Presentation40,
    ProposalFormat::Presentation45,
    ProposalFormat::Workshop120,
    ProposalFormat::Workshop240,
];

const SORT_FIELDS: &[ProposalSortBy] = &[
    ProposalSortBy::Created,
    ProposalSortBy::Title,
    ProposalSortBy::Speaker,
    ProposalSortBy::Rating,
    ProposalSortBy::Status,
];

const SORT_LABELS: &[&str] = &["Created", "Title", "Speaker", "Rating", "Status"];

pub async fn list_interactive(client: &TrpcClient, all_proposals: &[Proposal]) -> Result<()> {
    if all_proposals.is_empty() {
        println!("No proposals found.");
        return Ok(());
    }

    let hints = "↑↓ navigate · type to search · enter select · esc quit".dimmed();
    let mut filters = Filters::default();
    let mut cursor = 0usize;

    loop {
        let filtered = apply_filters(all_proposals, &filters);
        let summary = filter_summary(&filters);

        if filtered.is_empty() {
            println!("No proposals match current filters. Press enter to adjust filters.");
            show_filter_menu(&mut filters)?;
            continue;
        }

        let menu_label = format!("⚙ Filter & Sort  ({summary})");
        let mut items: Vec<String> = vec![menu_label];
        items.extend(filtered.iter().map(|p| format_item(p)));

        let default = (cursor + 1).min(items.len() - 1);

        let max_rows = ui::max_visible_items(&items, 4);

        let selection = FuzzySelect::new()
            .with_prompt(format!(
                "{}/{} proposals\n  {TABLE_HEADER}\n  {hints}",
                filtered.len(),
                all_proposals.len()
            ))
            .items(&items)
            .default(default)
            .max_length(max_rows)
            .highlight_matches(false)
            .interact_opt()?;

        match selection {
            Some(0) => {
                show_filter_menu(&mut filters)?;
                cursor = 0;
            }
            Some(idx) => {
                cursor = idx - 1;
                let proposal_ids: Vec<&str> = filtered.iter().map(|p| p.id.as_str()).collect();
                cursor = show_detail_loop(client, &proposal_ids, cursor).await?;
            }
            None => break,
        }
    }

    Ok(())
}

pub fn apply_filters<'a>(proposals: &'a [Proposal], filters: &Filters) -> Vec<&'a Proposal> {
    let mut filtered: Vec<&Proposal> = proposals
        .iter()
        .filter(|p| {
            if !filters.statuses.is_empty() && !filters.statuses.contains(&p.status) {
                return false;
            }
            if !filters.formats.is_empty() {
                if let Some(f) = p.format {
                    if !filters.formats.contains(&f) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            if let Some(search) = &filters.search {
                let search = search.to_lowercase();
                if !p.title.to_lowercase().contains(&search)
                    && !p
                        .speakers
                        .iter()
                        .any(|s| s.name.to_lowercase().contains(&search))
                {
                    return false;
                }
            }
            true
        })
        .collect();

    #[allow(clippy::cast_precision_loss)]
    filtered.sort_by(|a, b| {
        let cmp = match filters.sort_by {
            ProposalSortBy::Created => a.created_at.cmp(&b.created_at),
            ProposalSortBy::Title => a.title.cmp(&b.title),
            ProposalSortBy::Speaker => {
                let sa = a.speakers.first().map_or("", |s| &s.name);
                let sb = b.speakers.first().map_or("", |s| &s.name);
                sa.cmp(sb)
            }
            ProposalSortBy::Rating => {
                let ra = a
                    .reviews
                    .iter()
                    .filter_map(|r| r.score.as_ref())
                    .map(crate::types::ReviewScore::total)
                    .sum::<f64>()
                    / a.reviews.len().max(1) as f64;
                let rb = b
                    .reviews
                    .iter()
                    .filter_map(|r| r.score.as_ref())
                    .map(crate::types::ReviewScore::total)
                    .sum::<f64>()
                    / b.reviews.len().max(1) as f64;
                ra.partial_cmp(&rb).unwrap_or(std::cmp::Ordering::Equal)
            }
            ProposalSortBy::Status => a.status.to_string().cmp(&b.status.to_string()),
        };

        if filters.sort_order == SortOrder::Asc {
            cmp
        } else {
            cmp.reverse()
        }
    });

    filtered
}

pub fn show_filter_menu(filters: &mut Filters) -> Result<()> {
    use dialoguer::{MultiSelect, Select};

    let term = console::Term::stderr();
    term.clear_screen()?;

    // Status filter
    let status_defaults: Vec<bool> = STATUSES
        .iter()
        .map(|s| filters.statuses.contains(s))
        .collect();
    let status_labels: Vec<String> = STATUSES
        .iter()
        .map(std::string::ToString::to_string)
        .collect();

    println!(
        "{}",
        "Filter by status (space to toggle, enter to confirm):".bold()
    );
    let selected = MultiSelect::new()
        .items(&status_labels)
        .defaults(&status_defaults)
        .interact()?;
    filters.statuses = selected.iter().map(|&i| STATUSES[i]).collect();

    // Format filter
    let format_defaults: Vec<bool> = FORMATS
        .iter()
        .map(|f| {
            if filters.formats.is_empty() {
                true
            } else {
                filters.formats.contains(f)
            }
        })
        .collect();
    let format_labels: Vec<&str> = FORMATS.iter().map(|f: &ProposalFormat| f.label()).collect();

    println!(
        "\n{}",
        "Filter by format (space to toggle, enter to confirm):".bold()
    );
    let selected = MultiSelect::new()
        .items(&format_labels)
        .defaults(&format_defaults)
        .interact()?;
    filters.formats = if selected.len() == FORMATS.len() {
        vec![]
    } else {
        selected.iter().map(|&i| FORMATS[i]).collect()
    };

    // Sort field
    let sort_default = SORT_FIELDS
        .iter()
        .position(|&s| s == filters.sort_by)
        .unwrap_or(0);

    println!("\n{}", "Sort by:".bold());
    let sort_idx = Select::new()
        .items(SORT_LABELS)
        .default(sort_default)
        .interact()?;
    filters.sort_by = SORT_FIELDS[sort_idx];

    // Sort direction
    let dir_default = match filters.sort_order {
        SortOrder::Desc => 0,
        SortOrder::Asc => 1,
    };
    println!("\n{}", "Sort direction:".bold());
    let dir_idx = Select::new()
        .items(["Descending ↓", "Ascending ↑"])
        .default(dir_default)
        .interact()?;
    filters.sort_order = if dir_idx == 0 {
        SortOrder::Desc
    } else {
        SortOrder::Asc
    };

    term.clear_screen()?;
    Ok(())
}

async fn show_detail_loop(
    client: &TrpcClient,
    proposal_ids: &[&str],
    start: usize,
) -> Result<usize> {
    let reviewer_name = config::load().ok().and_then(|c| c.name);
    let mut idx = start;
    let total = proposal_ids.len();

    loop {
        let sp = ui::spinner("Loading…");
        let proposal = super::fetch_one(client, proposal_ids[idx]).await?;
        sp.finish_and_clear();

        let content = display::render_proposal_detail(&proposal);

        let mut nav = vec![];
        if idx > 0 {
            nav.push("← prev");
        }
        if idx + 1 < total {
            nav.push("→ next");
        }
        let mut nav_full = nav.clone();
        nav_full.extend(["↑↓/jk scroll", "^u/^d half-page", "r review", "q/esc back"]);
        let footer_measure = nav_full.join(" · ");

        let mut pager = ui::Pager::new(&content, &footer_measure);

        if pager.is_scrollable() {
            nav.push("↑↓/jk scroll");
            nav.push("^u/^d half-page");
        }
        nav.push("r review");
        nav.push("q/esc back");
        let footer = nav.join(" · ").dimmed().to_string();

        loop {
            let header = if pager.is_scrollable() {
                format!(
                    "[{}/{}] ↕ {}/{}",
                    idx + 1,
                    total,
                    pager.scroll_offset() + 1,
                    pager.line_count()
                )
            } else {
                format!("[{}/{}]", idx + 1, total)
            };

            pager.render(&header.dimmed().to_string(), &footer)?;

            match pager.handle_key()? {
                ui::pager::Action::Redraw => {}
                ui::pager::Action::Custom(key) => match key {
                    Key::ArrowLeft | Key::Char('h') => {
                        idx = idx.saturating_sub(1);
                        break;
                    }
                    Key::ArrowRight | Key::Char('l') => {
                        if idx + 1 < total {
                            idx += 1;
                        }
                        break;
                    }
                    Key::Char('r') => {
                        println!();
                        prompt_and_submit_review(client, &proposal, reviewer_name.as_deref())
                            .await?;
                        break;
                    }
                    Key::Escape | Key::Char('q') => {
                        pager.clear()?;
                        return Ok(idx);
                    }
                    _ => {}
                },
            }
        }
    }
}

pub async fn add_wizard(client: &TrpcClient) -> Result<()> {
    use dialoguer::{Input, MultiSelect, Select};

    println!("{}", "── New Proposal Wizard ──".bold().cyan());

    let title: String = Input::new().with_prompt("Proposal Title").interact_text()?;

    let description: String = Input::new()
        .with_prompt("Abstract / Description")
        .interact_text()?;

    let outline: String = Input::new()
        .with_prompt("Outline (internal)")
        .interact_text()?;

    let format_labels: Vec<&str> = FORMATS.iter().map(|f| f.label()).collect();
    let format_idx = Select::new()
        .with_prompt("Talk Format")
        .items(&format_labels)
        .default(3) // presentation_40
        .interact()?;
    let format = FORMATS[format_idx];

    let level: String = Input::new()
        .with_prompt("Technical Level (e.g. beginner, intermediate, expert)")
        .default("intermediate".into())
        .interact_text()?;

    let language: String = Input::new()
        .with_prompt("Language")
        .default("english".into())
        .interact_text()?;

    // Fetch speakers to allow selection
    let sp = ui::spinner("Fetching speakers…");
    let all_speakers = crate::commands::speakers::fetch_all(client).await?;
    sp.finish_and_clear();

    if all_speakers.is_empty() {
        anyhow::bail!("No speakers found in database. Create a speaker first.");
    }

    let speaker_labels: Vec<String> = all_speakers
        .iter()
        .map(|s| format!("{} <{}>", s.name, s.email.as_deref().unwrap_or("no email")))
        .collect();

    println!("\nSelect speakers (space to toggle, enter to confirm):");
    let chosen_indices = MultiSelect::new().items(&speaker_labels).interact()?;

    if chosen_indices.is_empty() {
        anyhow::bail!("At least one speaker is required.");
    }

    let speaker_ids: Vec<String> = chosen_indices
        .iter()
        .map(|&i| all_speakers[i].id.clone())
        .collect();

    let sp = ui::spinner("Creating proposal…");
    let input = super::args::CreateArgs {
        title,
        format: Some(format),
        level: Some(level),
        language: Some(language),
        speakers: Some(speaker_ids),
        audiences: Some(vec!["developer".into()]), // Default for wizard
        topics: Some(vec![]),                      // Logic below will override this
        tos: true,
        description: Some(description),
        outline: Some(outline),
    };

    let proposal: Proposal = client
        .mutate("proposal.admin.create", &serde_json::to_value(input)?)
        .await?;
    sp.finish_and_clear();

    println!(
        "{} Successfully created proposal {} (ID: {})",
        "✓".green(),
        proposal.title.bold(),
        proposal.id.dimmed()
    );

    Ok(())
}
