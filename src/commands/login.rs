use anyhow::Result;
use colored::Colorize;
use dialoguer::Select;

use crate::auth;
use crate::config::{self, Config};

const CONFERENCES: &[(&str, &str)] = &[
    ("2026.cloudnativedays.no", "https://2026.cloudnativedays.no"),
    (
        "2025.cloudnativebergen.dev",
        "https://2025.cloudnativebergen.dev",
    ),
    (
        "2024.cloudnativebergen.dev",
        "https://2024.cloudnativebergen.dev",
    ),
];

pub fn run(url_arg: Option<String>) -> Result<()> {
    if config::exists() {
        let cfg = config::load()?;
        println!(
            "Already logged in to {}. Run `konf logout` first to switch.",
            cfg.conference_title
        );
        return Ok(());
    }

    let (title, url) = if let Some(u) = url_arg {
        (u.clone(), u)
    } else {
        let mut items: Vec<&str> = CONFERENCES.iter().map(|(title, _)| *title).collect();
        items.push("Custom URL...");

        let selection = Select::new()
            .with_prompt("Select conference")
            .items(&items)
            .default(0)
            .interact()?;

        if selection < CONFERENCES.len() {
            let (t, u) = CONFERENCES[selection];
            (t.to_string(), u.to_string())
        } else {
            let custom_url: String = dialoguer::Input::new()
                .with_prompt("Enter conference URL (e.g., http://localhost:3000)")
                .interact_text()?;
            (custom_url.clone(), custom_url)
        }
    };

    let result = auth::browser_login(&url)?;

    let cfg = Config {
        api_url: url.clone(),
        token: result.token,
        conference_id: result.conference_id.unwrap_or_default(),
        conference_title: title.clone(),
        name: result.name.clone(),
    };
    config::save(&cfg)?;

    println!();
    if let Some(name) = &result.name {
        println!("{} Welcome, {}!", "✓".green().bold(), name.bold());
    } else {
        println!("{} Authenticated successfully!", "✓".green().bold());
    }
    println!("  Conference: {}", title.cyan());
    println!();
    println!(
        "  Run {} to see available commands.",
        "konf admin --help".dimmed()
    );
    Ok(())
}
