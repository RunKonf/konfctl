use anyhow::Result;
use colored::Colorize;
use console::Key;
use dialoguer::{FuzzySelect, Input, MultiSelect, Select};

use crate::client::TrpcClient;
use crate::display;
use crate::types::{ActivityType, SortOrder, SponsorSortBy, SponsorStatus};
use crate::ui;

use super::{EmailArgs, ListArgs, NoteArgs};

const STATUSES: &[SponsorStatus] = &[
    SponsorStatus::Prospect,
    SponsorStatus::Contacted,
    SponsorStatus::Negotiating,
    SponsorStatus::ClosedWon,
    SponsorStatus::ClosedLost,
];

const SORT_FIELDS: &[SponsorSortBy] = &[
    SponsorSortBy::LastActivity,
    SponsorSortBy::FollowUp,
    SponsorSortBy::Stale,
    SponsorSortBy::Value,
    SponsorSortBy::Name,
    SponsorSortBy::CreatedAt,
];

pub async fn list_interactive(client: &TrpcClient, initial_args: ListArgs) -> Result<()> {
    let hints = "↑↓ navigate · type to search · enter select · esc quit".dimmed();
    let mut args = initial_args;
    let mut cursor = 0usize;

    loop {
        let sp = ui::spinner("Fetching sponsors…");
        let sponsors = super::fetch_all(client, &args).await?;
        sp.finish_and_clear();

        if sponsors.is_empty() {
            println!("No sponsors match current filters. Press enter to adjust filters.");
            show_filter_menu(&mut args)?;
            cursor = 0;
            continue;
        }

        let mut summary_parts = vec![];
        if let Some(statuses) = &args.status
            && !statuses.is_empty()
        {
            summary_parts.push(format!("status: {}", statuses.len()));
        }
        if args.mine {
            summary_parts.push("mine".to_string());
        }
        if args.unassigned {
            summary_parts.push("unassigned".to_string());
        }
        if args.due {
            summary_parts.push("due".to_string());
        }
        if args.has_follow_up {
            summary_parts.push("has-follow-up".to_string());
        }
        if let Some(stale) = args.stale_days {
            summary_parts.push(format!("stale: >{stale}d"));
        }

        let summary = if summary_parts.is_empty() {
            "all".to_string()
        } else {
            summary_parts.join(", ")
        };

        let menu_label = format!("⚙ Filter & Sort  ({summary})");
        let mut items: Vec<String> = vec![menu_label];
        items.extend(sponsors.iter().map(display::format_sponsor_row));

        let default = (cursor + 1).min(items.len() - 1);

        let max_rows = ui::max_visible_items(&items, 4);

        let selection = FuzzySelect::new()
            .with_prompt(format!(
                "{} sponsors\n  {}\n  {hints}",
                sponsors.len(),
                display::SPONSOR_TABLE_HEADER,
            ))
            .items(&items)
            .default(default)
            .max_length(max_rows)
            .highlight_matches(false)
            .interact_opt()?;

        match selection {
            Some(0) => {
                show_filter_menu(&mut args)?;
                cursor = 0;
            }
            Some(idx) => {
                cursor = idx - 1;
                let ids: Vec<&str> = sponsors.iter().map(|s| s.id.as_str()).collect();
                cursor = show_detail_loop(client, &ids, cursor).await?;
            }
            None => break,
        }
    }

    Ok(())
}

pub fn show_filter_menu(args: &mut ListArgs) -> Result<()> {
    let term = console::Term::stderr();
    term.clear_screen()?;

    // Status filter
    let status_defaults: Vec<bool> = STATUSES
        .iter()
        .map(|s| {
            args.status
                .as_ref()
                .is_some_and(|statuses| statuses.contains(s))
        })
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

    args.status = if selected.is_empty() {
        None
    } else {
        Some(selected.iter().map(|&i| STATUSES[i]).collect())
    };

    // Sort field
    let sort_default = args
        .sort_by
        .and_then(|sb| SORT_FIELDS.iter().position(|&s| s == sb))
        .unwrap_or(0);

    let sort_labels: Vec<String> = SORT_FIELDS.iter().map(|s| format!("{s:?}")).collect();

    println!("\n{}", "Sort by:".bold());
    let sort_idx = Select::new()
        .items(&sort_labels)
        .default(sort_default)
        .interact()?;
    args.sort_by = Some(SORT_FIELDS[sort_idx]);

    // Sort direction
    let dir_default = match args.sort_order.unwrap_or(SortOrder::Desc) {
        SortOrder::Desc => 0,
        SortOrder::Asc => 1,
    };
    println!("\n{}", "Sort direction:".bold());
    let dir_idx = Select::new()
        .items(["Descending ↓", "Ascending ↑"])
        .default(dir_default)
        .interact()?;
    args.sort_order = Some(if dir_idx == 0 {
        SortOrder::Desc
    } else {
        SortOrder::Asc
    });

    term.clear_screen()?;
    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn show_detail_loop(client: &TrpcClient, ids: &[&str], start: usize) -> Result<usize> {
    let mut idx = start;
    let total = ids.len();

    loop {
        let sp = ui::spinner("Loading…");
        let sponsor = super::fetch_one(client, ids[idx]).await?;
        sp.finish_and_clear();

        let content = display::render_sponsor_detail(&sponsor);

        let mut nav = vec![];
        if idx > 0 {
            nav.push("← prev");
        }
        if idx + 1 < total {
            nav.push("→ next");
        }
        let mut nav_full = nav.clone();
        nav_full.extend([
            "↑↓/jk scroll",
            "^u/^d half-page",
            "m move stage",
            "a assign",
            "n add note",
            "e email",
            "d delete log",
            "q/esc back",
        ]);
        let footer_measure = nav_full.join(" · ");

        let mut pager = ui::Pager::new(&content, &footer_measure);

        if pager.is_scrollable() {
            nav.push("↑↓/jk scroll");
            nav.push("^u/^d half-page");
        }
        nav.push("m move stage");
        nav.push("a assign");
        nav.push("n add note");
        nav.push("e email");
        nav.push("d delete log");
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
                    Key::Char('m') => {
                        println!();
                        let labels: Vec<String> = STATUSES
                            .iter()
                            .map(std::string::ToString::to_string)
                            .collect();

                        let current_idx = STATUSES
                            .iter()
                            .position(|&s| s == sponsor.status)
                            .unwrap_or(0);

                        if let Some(selection) = Select::new()
                            .with_prompt("Move to stage")
                            .items(&labels)
                            .default(current_idx)
                            .interact_opt()?
                        {
                            super::move_stage(&sponsor.id, STATUSES[selection]).await?;
                        }
                        break;
                    }
                    Key::Char('a') => {
                        println!();
                        let sp = ui::spinner("Fetching organizers…");
                        let organizers = super::fetch_organizers(client).await?;
                        sp.finish_and_clear();

                        if organizers.is_empty() {
                            println!("No organizers found in conference settings.");
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            break;
                        }

                        let mut labels: Vec<String> = vec!["<Unassigned>".to_string()];
                        labels.extend(organizers.iter().map(|o| o.name.clone()));

                        let default_idx = sponsor.assigned_to.as_ref().and_then(|assigned| {
                            organizers
                                .iter()
                                .position(|o| o.id == assigned.id)
                                .map(|pos| pos + 1)
                        });

                        let selection = Select::new()
                            .with_prompt("Assign Organizer")
                            .items(&labels)
                            .default(default_idx.unwrap_or(0))
                            .interact_opt()?;

                        if let Some(idx) = selection {
                            let speaker_id = if idx == 0 {
                                None
                            } else {
                                Some(organizers[idx - 1].id.as_str())
                            };
                            super::assign(&sponsor.id, speaker_id).await?;
                        }
                        break;
                    }
                    Key::Char('n') => {
                        println!();
                        let note: String = Input::new().with_prompt("Add Note").interact_text()?;
                        if !note.is_empty() {
                            let note_args = NoteArgs {
                                id: sponsor.id.clone(),
                                kind: ActivityType::Note,
                                description: note,
                            };
                            super::add_note(note_args).await?;
                        }
                        break;
                    }
                    Key::Char('e') => {
                        println!();
                        let email_args = EmailArgs {
                            id: sponsor.id.clone(),
                            template: None,
                            subject: None,
                            message: None,
                            edit: false,
                            dry_run: false,
                            json: false,
                        };
                        super::email::run(email_args).await?;
                        break;
                    }
                    Key::Char('d') => {
                        println!();
                        let sp = ui::spinner("Fetching history…");
                        let activities = super::fetch_activities(client, &sponsor.id).await?;
                        sp.finish_and_clear();

                        if activities.is_empty() {
                            println!("No activities to delete.");
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            break;
                        }

                        // Filter for deletable activities (note, email, call, meeting)
                        let deletable: Vec<_> = activities
                            .iter()
                            .filter(|a| {
                                matches!(
                                    a.activity_type,
                                    ActivityType::Note
                                        | ActivityType::Email
                                        | ActivityType::Call
                                        | ActivityType::Meeting
                                )
                            })
                            .collect();

                        if deletable.is_empty() {
                            println!(
                                "No user-supplied activities to delete (system logs are protected)."
                            );
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            break;
                        }

                        let labels: Vec<String> = deletable
                            .iter()
                            .map(|a| {
                                format!(
                                    "[{}] {} - {}...",
                                    a.activity_type,
                                    &a.created_at[..10],
                                    &a.description[..std::cmp::min(40, a.description.len())]
                                )
                            })
                            .collect();

                        let selection = FuzzySelect::new()
                            .with_prompt("Select activity to delete")
                            .items(&labels)
                            .interact_opt()?;

                        if let Some(idx) = selection {
                            let activity = deletable[idx];
                            if dialoguer::Confirm::new()
                                .with_prompt(format!(
                                    "Are you sure you want to delete this {}?",
                                    activity.activity_type
                                ))
                                .interact()?
                            {
                                super::delete_activity(&activity.id, true).await?;
                            }
                        }
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
