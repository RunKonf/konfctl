use std::io::Write;

use anyhow::{Context, Result, bail};
use colored::Colorize;
use dialoguer::{Confirm, FuzzySelect, Select};

use super::args::EmailArgs;
use crate::client::TrpcClient;
use crate::template;
use crate::types::{SendEmailResponse, SponsorEmailTemplate, TemplateListResponse};
use crate::ui;

pub async fn run(args: EmailArgs) -> Result<()> {
    let client = super::require_client()?;

    let sp = ui::spinner("Fetching templates…");
    let template_resp = fetch_templates(&client, &args.id).await?;
    sp.finish_and_clear();

    let sponsor_name = template_resp.sponsor_name.as_deref().unwrap_or("Unknown");

    let is_interactive = console::Term::stdout().is_term() && args.message.is_none();

    // Resolve subject and body
    let (subject, mut body) = if let Some(ref message) = args.message {
        let subject = args
            .subject
            .clone()
            .unwrap_or_else(|| format!("Message to {sponsor_name}"));
        (subject, message.clone())
    } else if is_interactive {
        pick_template_interactive(&args, &template_resp, sponsor_name)?
    } else {
        let slug = args.template.as_deref().ok_or_else(|| {
            anyhow::anyhow!("--template or --message required in non-interactive mode")
        })?;
        pick_template_by_slug(&args, &template_resp, slug)?
    };

    // Open $EDITOR if --edit flag is set (non-interactive edit-before-send)
    if args.edit {
        body = edit_in_editor(&body)?;
    }

    // Warn about unresolved variables
    if !warn_unresolved(&subject, &body, args.dry_run, is_interactive)? {
        return Ok(());
    }

    // Show preview
    let recipients_display = format_recipients(&template_resp);

    if !args.json {
        print_preview(&subject, &body, &recipients_display);
    }

    if args.dry_run {
        return print_dry_run(&args, &subject, &body, &template_resp);
    }

    // Confirm before sending (interactive only)
    if is_interactive {
        let action = Select::new()
            .with_prompt("Action")
            .items(["Send", "Edit in $EDITOR", "Cancel"])
            .default(0)
            .interact()?;

        match action {
            0 => {} // Send
            1 => {
                body = edit_in_editor(&body)?;
            }
            _ => {
                println!("Cancelled.");
                return Ok(());
            }
        }
    }

    send_email(
        &client,
        &args,
        &args.id,
        &subject,
        &body,
        &recipients_display,
    )
    .await
}

fn pick_template_interactive(
    args: &EmailArgs,
    resp: &TemplateListResponse,
    sponsor_name: &str,
) -> Result<(String, String)> {
    if resp.templates.is_empty() {
        bail!("No email templates found. Create templates in the web UI first.");
    }

    let items: Vec<String> = resp.templates.iter().map(format_template_item).collect();

    let default_idx = if let Some(ref slug) = args.template {
        resp.templates
            .iter()
            .position(|t| t.slug.current == *slug)
            .unwrap_or(0)
    } else {
        0 // Templates arrive pre-sorted by relevance
    };

    let hints = "↑↓ navigate · type to search · enter select · esc cancel".dimmed();
    let max_rows = ui::max_visible_items(&items, 4);

    let selection = FuzzySelect::new()
        .with_prompt(format!(
            "Email template for {}\n  {hints}",
            sponsor_name.bold()
        ))
        .items(&items)
        .default(default_idx)
        .max_length(max_rows)
        .highlight_matches(false)
        .interact_opt()?;

    let idx = selection.ok_or_else(|| anyhow::anyhow!("Cancelled"))?;
    let tmpl = &resp.templates[idx];

    Ok(resolve_template(args, tmpl, &resp.variables))
}

fn pick_template_by_slug(
    args: &EmailArgs,
    resp: &TemplateListResponse,
    slug: &str,
) -> Result<(String, String)> {
    let tmpl = resp
        .templates
        .iter()
        .find(|t| t.slug.current == slug)
        .ok_or_else(|| anyhow::anyhow!("Template not found: {slug}"))?;

    Ok(resolve_template(args, tmpl, &resp.variables))
}

fn resolve_template(
    args: &EmailArgs,
    tmpl: &SponsorEmailTemplate,
    variables: &std::collections::HashMap<String, String>,
) -> (String, String) {
    let subject = args
        .subject
        .clone()
        .unwrap_or_else(|| template::substitute_variables(&tmpl.subject, variables));

    let raw_body = tmpl.body_markdown.as_deref().unwrap_or("");
    let body = template::substitute_variables(raw_body, variables);

    (subject, body)
}

fn format_template_item(t: &SponsorEmailTemplate) -> String {
    let lang = match t.language {
        crate::types::TemplateLanguage::Norwegian => "🇳🇴",
        crate::types::TemplateLanguage::English => "🇬🇧",
        crate::types::TemplateLanguage::Unknown => "  ",
    };
    let default_marker = if t.is_default == Some(true) {
        " ★"
    } else {
        ""
    };
    format!(
        "{} {:<30} {:<16}{}",
        lang,
        ui::truncate(&t.title, 28),
        t.category,
        default_marker
    )
}

/// Warn about unresolved template variables.
/// Returns `true` if the caller should proceed, `false` if the user cancelled.
fn warn_unresolved(subject: &str, body: &str, dry_run: bool, is_interactive: bool) -> Result<bool> {
    let unresolved_subject = template::find_unresolved_variables(subject);
    let unresolved_body = template::find_unresolved_variables(body);
    let all_unresolved: Vec<&str> = unresolved_subject
        .iter()
        .chain(unresolved_body.iter())
        .map(String::as_str)
        .collect();

    if all_unresolved.is_empty() {
        return Ok(true);
    }

    eprintln!(
        "{} Unresolved template variables: {}",
        "⚠".yellow(),
        all_unresolved.join(", ").yellow()
    );

    if !dry_run && is_interactive {
        let proceed = Confirm::new()
            .with_prompt("Send anyway?")
            .default(false)
            .interact()?;
        if !proceed {
            println!("Cancelled.");
            return Ok(false);
        }
    }
    Ok(true)
}

fn format_recipients(resp: &TemplateListResponse) -> Vec<String> {
    resp.recipients
        .iter()
        .map(|r| format!("{} <{}>", r.name, r.email))
        .collect()
}

fn print_preview(subject: &str, body: &str, recipients_display: &[String]) {
    println!();
    println!("{}", "── Email Preview ──".dimmed());
    println!("  {} {}", "To:".bold(), recipients_display.join(", "));
    println!("  {} {}", "Subject:".bold(), subject);
    println!("{}", "───────────────────".dimmed());
    for line in body.lines() {
        println!("  {line}");
    }
    println!("{}", "───────────────────".dimmed());
    println!();
}

fn print_dry_run(
    args: &EmailArgs,
    subject: &str,
    body: &str,
    template_resp: &TemplateListResponse,
) -> Result<()> {
    if args.json {
        let preview = serde_json::json!({
            "dryRun": true,
            "subject": subject,
            "body": body,
            "recipients": template_resp.recipients,
        });
        println!("{}", serde_json::to_string_pretty(&preview)?);
    } else {
        println!("{}", "Dry run — email not sent.".dimmed());
    }
    Ok(())
}

fn edit_in_editor(body: &str) -> Result<String> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(target_os = "macos") {
                "nano".to_string()
            } else {
                "vi".to_string()
            }
        });

    let mut tmp = tempfile::Builder::new()
        .prefix("konf-email-")
        .suffix(".md")
        .tempfile()
        .context("Failed to create temporary file")?;

    tmp.write_all(body.as_bytes())
        .context("Failed to write to temporary file")?;
    tmp.flush()?;

    let path = tmp.path().to_path_buf();

    #[allow(clippy::needless_borrows_for_generic_args)]
    let status = std::process::Command::new(&editor)
        .arg(&path)
        .status()
        .with_context(|| format!("Failed to launch editor: {editor}"))?;

    if !status.success() {
        bail!("Editor exited with non-zero status");
    }

    let edited = std::fs::read_to_string(&path).context("Failed to read edited file")?;
    Ok(edited)
}

pub async fn fetch_templates(client: &TrpcClient, sfc_id: &str) -> Result<TemplateListResponse> {
    let input = serde_json::json!({ "sponsorForConferenceId": sfc_id });
    client
        .query("sponsor.emailTemplates.listForSponsor", Some(&input))
        .await
}

async fn send_email(
    client: &TrpcClient,
    args: &EmailArgs,
    sfc_id: &str,
    subject: &str,
    body: &str,
    recipients_display: &[String],
) -> Result<()> {
    let sp = ui::spinner("Sending email…");
    let input = serde_json::json!({
        "sponsorForConferenceId": sfc_id,
        "subject": subject,
        "body": body,
    });

    let result: SendEmailResponse = client.mutate("sponsor.crm.sendEmailBySfc", &input).await?;
    sp.finish_and_clear();

    if args.json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!(
            "{} Email sent to {} recipient(s): {}",
            "✓".green().bold(),
            result.recipient_count.unwrap_or(0),
            recipients_display.join(", ")
        );
        if let Some(ref email_id) = result.email_id {
            println!("  Email ID: {}", email_id.dimmed());
        }
    }

    Ok(())
}
