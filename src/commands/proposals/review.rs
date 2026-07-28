use anyhow::Result;
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};

use crate::client::TrpcClient;
use crate::types::{Proposal, ReviewInput, ReviewScore};
use crate::ui;

pub async fn prompt_and_submit_review(
    client: &TrpcClient,
    proposal: &Proposal,
    reviewer_name: Option<&str>,
) -> Result<()> {
    // Find the user's existing review to pre-fill defaults
    let existing = reviewer_name.and_then(|name| {
        proposal.reviews.iter().find(|r| {
            r.reviewer
                .as_ref()
                .is_some_and(|rev| rev.name.eq_ignore_ascii_case(name))
        })
    });

    if existing.is_some() {
        println!("{}", "Updating your existing review.".dimmed());
    }

    let prev_score = existing.and_then(|r| r.score.as_ref());
    let prev_comment = existing.and_then(|r| r.comment.as_str()).unwrap_or("");

    // Prompt scores (Esc to cancel at any step)
    let Some(content) = prompt_score("Content", score_default(prev_score, |s| s.content))? else {
        println!("{}", "Review cancelled.".dimmed());
        return Ok(());
    };
    let Some(relevance) = prompt_score("Relevance", score_default(prev_score, |s| s.relevance))?
    else {
        println!("{}", "Review cancelled.".dimmed());
        return Ok(());
    };
    let Some(speaker) = prompt_score("Speaker", score_default(prev_score, |s| s.speaker))? else {
        println!("{}", "Review cancelled.".dimmed());
        return Ok(());
    };

    let comment: String = Input::new()
        .with_prompt("Comment")
        .with_initial_text(prev_comment)
        .allow_empty(true)
        .interact_text()?;

    // Show summary and confirm
    let total = f64::from(content) + f64::from(relevance) + f64::from(speaker);
    println!(
        "\n  Content: {content}  Relevance: {relevance}  Speaker: {speaker}  Total: {total:.0}/15"
    );
    if !comment.is_empty() {
        println!("  Comment: {comment}");
    }

    if !Confirm::new()
        .with_prompt("Submit review?")
        .default(true)
        .interact()?
    {
        println!("{}", "Review cancelled.".dimmed());
        return Ok(());
    }

    let input = ReviewInput {
        id: proposal.id.clone(),
        comment,
        score: ReviewScore {
            content: f64::from(content),
            relevance: f64::from(relevance),
            speaker: f64::from(speaker),
        },
    };

    let sp = ui::spinner("Submitting review…");
    super::submit_review(client, &input).await?;
    sp.finish_and_clear();

    println!(
        "{} Review submitted ({:.0}/15)",
        "✔".green().bold(),
        input.score.total()
    );
    Ok(())
}

fn score_default(prev: Option<&ReviewScore>, f: impl Fn(&ReviewScore) -> f64) -> usize {
    prev.map_or(2, |s| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let v = f(s) as usize;
        v.saturating_sub(1).min(4)
    })
}

const SCORE_LABELS: &[&str] = &[
    "1 - Poor",
    "2 - Fair",
    "3 - Good",
    "4 - Very good",
    "5 - Excellent",
];

fn prompt_score(category: &str, default: usize) -> Result<Option<u8>> {
    let selection = Select::new()
        .with_prompt(format!("{category} (1–5, esc to cancel)"))
        .items(SCORE_LABELS)
        .default(default)
        .interact_opt()?;
    #[allow(clippy::cast_possible_truncation)]
    Ok(selection.map(|idx| (idx + 1) as u8))
}
