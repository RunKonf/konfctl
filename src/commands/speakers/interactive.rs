use anyhow::Result;
use colored::Colorize;
use console::Key;
use dialoguer::FuzzySelect;
use std::fmt::Write;

use crate::client::TrpcClient;
use crate::types::SpeakerSummary;
use crate::ui;

pub async fn list_interactive(
    client: &TrpcClient,
    initial_speakers: &[SpeakerSummary],
) -> Result<()> {
    if initial_speakers.is_empty() {
        println!("No speakers found.");
        return Ok(());
    }

    let hints = "↑↓ navigate · type to search · enter select · esc quit".dimmed();
    let mut search: Option<String> = None;
    let mut cursor = 0usize;

    loop {
        let filtered: Vec<&SpeakerSummary> = initial_speakers
            .iter()
            .filter(|s| {
                if let Some(ref q) = search {
                    let q = q.to_lowercase();
                    s.name.to_lowercase().contains(&q)
                        || s.email
                            .as_ref()
                            .is_some_and(|e| e.to_lowercase().contains(&q))
                } else {
                    true
                }
            })
            .collect();

        let items: Vec<String> = filtered
            .iter()
            .map(|s| {
                format!(
                    "{:<20} {:<20} {}",
                    s.id,
                    s.name,
                    s.email.as_deref().unwrap_or_default()
                )
            })
            .collect();

        if items.is_empty() && search.is_some() {
            println!("No speakers match current filters. Clearing search.");
            search = None;
            continue;
        }

        let max_rows = ui::max_visible_items(&items, 4);

        let selection = FuzzySelect::new()
            .with_prompt(format!(
                "{} speakers\n  {}\n  {hints}",
                filtered.len(),
                "ID                   NAME                 EMAIL"
                    .bold()
                    .cyan()
            ))
            .items(&items)
            .default(cursor)
            .max_length(max_rows)
            .highlight_matches(false)
            .interact_opt()?;

        match selection {
            Some(idx) => {
                cursor = idx;
                let speaker_id = &filtered[idx].id;
                show_detail_loop(client, speaker_id).await?;
            }
            None => break,
        }
    }

    Ok(())
}

async fn show_detail_loop(client: &TrpcClient, speaker_id: &str) -> Result<()> {
    let sp = ui::spinner("Loading speaker details…");
    let speaker = super::fetch_one(client, speaker_id).await?;
    let speaker_talks = super::fetch_talks_for_speaker(client, speaker_id).await?;
    sp.finish_and_clear();

    // Standard detail view logic from mod.rs but rendered to string for pager
    let mut content = format!("{} ({})\n", speaker.name.bold(), speaker.id.dimmed());
    if let Some(email) = speaker.email.as_str() {
        let _ = writeln!(content, "Email:   {email}");
    }
    if let Some(company) = speaker.company.as_str() {
        let _ = writeln!(content, "Company: {company}");
    }
    if let Some(title) = speaker.title.as_str() {
        let _ = writeln!(content, "Title:   {title}");
    }
    if !speaker.flags.is_empty() {
        let _ = writeln!(content, "Flags:   {:?}", speaker.flags);
    }
    if !speaker.links.is_empty() {
        let _ = writeln!(content, "\nLinks:");
        for link in &speaker.links {
            let _ = writeln!(content, "  - {link}");
        }
    }

    if !speaker_talks.is_empty() {
        let _ = writeln!(content, "\nTalks:");
        for talk in speaker_talks {
            let status = format!("[{}]", talk.status);
            let _ = writeln!(content, "  - {} {}", status.yellow(), talk.title);
        }
    }

    if !speaker.bio.is_null() {
        let _ = writeln!(content, "\nBio:");
        if speaker.bio.is_array() {
            let _ = writeln!(
                content,
                "{}",
                crate::types::portable_text_to_plain(&speaker.bio.as_array().unwrap().clone())
            );
        } else if speaker.bio.is_string() {
            let _ = writeln!(content, "{}", speaker.bio.as_str().unwrap());
        } else {
            let _ = writeln!(content, "{}", speaker.bio);
        }
    }

    let footer = "q/esc back".dimmed().to_string();
    let mut pager = ui::Pager::new(&content, &footer);

    loop {
        pager.render(
            &format!("Speaker: {}", speaker.name).bold().to_string(),
            &footer,
        )?;

        match pager.handle_key()? {
            ui::pager::Action::Redraw => {}
            ui::pager::Action::Custom(key) => match key {
                Key::Escape | Key::Char('q') => {
                    pager.clear()?;
                    return Ok(());
                }
                _ => {}
            },
        }
    }
}

pub async fn add_wizard(client: &TrpcClient) -> Result<()> {
    use crate::types::SpeakerFlag;
    use dialoguer::{Input, MultiSelect};

    println!("{}", "── New Speaker Wizard ──".bold().cyan());

    let name: String = Input::new().with_prompt("Full Name").interact_text()?;

    let email: String = Input::new()
        .with_prompt("Email Address")
        .validate_with(|input: &String| {
            if input.contains('@') && input.contains('.') {
                Ok(())
            } else {
                Err("Invalid email format")
            }
        })
        .interact_text()?;

    let company: String = Input::new()
        .with_prompt("Company (optional)")
        .allow_empty(true)
        .interact_text()?;

    let title: String = Input::new()
        .with_prompt("Job Title (optional)")
        .allow_empty(true)
        .interact_text()?;

    let bio: String = Input::new()
        .with_prompt("Bio (optional, plain text)")
        .allow_empty(true)
        .interact_text()?;

    let flags_options = &[
        SpeakerFlag::Local,
        SpeakerFlag::FirstTime,
        SpeakerFlag::Diverse,
        SpeakerFlag::RequiresFunding,
        SpeakerFlag::Keynote,
        SpeakerFlag::Hidden,
        SpeakerFlag::Internal,
    ];

    let flags_labels: Vec<String> = flags_options
        .iter()
        .map(std::string::ToString::to_string)
        .collect();

    println!("\nSelect flags (space to toggle, enter to confirm):");
    let chosen_indices = MultiSelect::new().items(&flags_labels).interact()?;

    let chosen_flags: Vec<SpeakerFlag> = chosen_indices.iter().map(|&i| flags_options[i]).collect();

    let sp = ui::spinner("Creating speaker profile…");
    let input = crate::types::SpeakerCreateInput {
        name,
        email,
        company: if company.is_empty() {
            None
        } else {
            Some(company)
        },
        title: if title.is_empty() { None } else { Some(title) },
        bio: if bio.is_empty() { None } else { Some(bio) },
        image: None,
        links: None,
        flags: Some(chosen_flags),
    };

    let speaker: crate::types::Speaker = client
        .mutate("speaker.admin.create", &serde_json::to_value(input)?)
        .await?;
    sp.finish_and_clear();

    println!(
        "{} Successfully created speaker {} (ID: {})",
        "✓".green(),
        speaker.name.bold(),
        speaker.id.dimmed()
    );

    Ok(())
}
