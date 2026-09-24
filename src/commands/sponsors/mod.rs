pub fn find_sponsor_by_name<'a>(
    sponsors: &'a [SponsorForConference],
    name: &str,
) -> Option<Vec<&'a SponsorForConference>> {
    let lower_input = name.to_lowercase();
    let matches: Vec<_> = sponsors
        .iter()
        .filter(|s| {
            s.sponsor
                .as_ref()
                .is_some_and(|sp| sp.name.to_lowercase() == lower_input)
        })
        .collect();

    if matches.is_empty() {
        None
    } else {
        Some(matches)
    }
}

mod args;
pub mod email;
mod interactive;

pub use args::{CreateArgs, EmailArgs, ListArgs, NoteArgs, UpdateArgs, UpdateContactsArgs};

use anyhow::{Context, Result};

use super::require_client;
use crate::client::TrpcClient;
use crate::display;
use crate::types::SponsorForConference;

// ── API helpers ──────────────────────────────────────────────────────────────

async fn resolve_id(client: &crate::client::TrpcClient, id_or_name: &str) -> Result<String> {
    // 1. Try to fetch it directly as an ID. If it succeeds, it's an ID!
    if let Ok(sponsor) = fetch_one(client, id_or_name).await {
        return Ok(sponsor.id.clone());
    }

    // 2. If it fails, maybe it's a name. Fetch all and search by name.
    let all = fetch_all(
        client,
        &ListArgs {
            view: None,
            search: None,
            status: None,
            mine: false,
            assigned_to: None,
            unassigned: false,
            tags: None,
            tiers: None,
            sort_by: None,
            sort_order: None,
            stale_days: None,
            due: false,
            has_follow_up: false,
            has_contact: false,
            compact: false,
            limit: None,
            json: false,
        },
    )
    .await?;

    let matches = find_sponsor_by_name(&all, id_or_name).unwrap_or_default();

    match matches.len() {
        1 => Ok(matches[0].id.clone()),
        0 => anyhow::bail!("No sponsor found matching '{id_or_name}' (and it is not a valid ID)"),
        _ => anyhow::bail!("Multiple sponsors found matching '{id_or_name}'. Please use exact ID."),
    }
}

pub async fn fetch_all(client: &TrpcClient, args: &ListArgs) -> Result<Vec<SponsorForConference>> {
    let sponsors: Vec<SponsorForConference> = client
        .query("sponsor.crm.list", Some(&serde_json::to_value(args)?))
        .await?;
    Ok(sponsors)
}

pub async fn fetch_one(client: &TrpcClient, id: &str) -> Result<SponsorForConference> {
    let sponsor: SponsorForConference = client
        .query(
            "sponsor.crm.getById",
            Some(&serde_json::json!({ "id": id })),
        )
        .await?;
    Ok(sponsor)
}

pub async fn fetch_activities(
    client: &TrpcClient,
    id: &str,
) -> Result<Vec<crate::types::SponsorActivity>> {
    let activities: Vec<crate::types::SponsorActivity> = client
        .query(
            "sponsor.crm.activities.list",
            Some(&serde_json::json!({ "sponsorForConferenceId": id })),
        )
        .await?;
    Ok(activities)
}

// ── Command entry points ─────────────────────────────────────────────────────

pub async fn list(args: ListArgs) -> Result<()> {
    let client = require_client()?;

    let all = fetch_all(&client, &args).await?;
    let unhandled =
        crate::display::print_json_list(all, args.limit, args.compact, args.json, |s| {
            serde_json::json!({
                "id": s.id,
                "name": s.sponsor.as_ref().map(|sp| &sp.name),
                "status": s.status,
                "tier": s.tier.as_ref().map(|t| &t.title),
                "contractStatus": s.contract_status,
            })
        })?;

    if let Some(all) = unhandled {
        if args.search.is_some()
            || args.status.is_some()
            || args.assigned_to.is_some()
            || args.unassigned
            || args.tags.is_some()
            || args.tiers.is_some()
            || args.sort_by.is_some()
            || args.sort_order.is_some()
            || args.stale_days.is_some()
            || args.due
            || args.has_follow_up
            || args.has_contact
            || !console::Term::stdout().is_term()
        {
            if all.is_empty() {
                println!("No sponsors match the given filters.");
            } else {
                println!("{}", display::SPONSOR_TABLE_HEADER);
                for s in &all {
                    println!("{}", display::format_sponsor_row(s));
                }
                println!("\n{} sponsors", all.len());
            }
        } else {
            interactive::list_interactive(&client, args).await?;
        }
    }
    Ok(())
}

pub async fn get(id: &str, json: bool) -> Result<()> {
    let client = require_client()?;
    let resolved_id = resolve_id(&client, id).await?;
    let id = &resolved_id;
    let sponsor = fetch_one(&client, id).await?;
    if json || crate::is_agent() {
        if crate::is_agent() {
            println!("{}", serde_json::to_string(&sponsor)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&sponsor)?);
        }
    } else {
        display::print_sponsor_detail(&sponsor);
    }
    Ok(())
}

pub async fn update(args: UpdateArgs) -> Result<()> {
    let client = require_client()?;
    let resolved_id = resolve_id(&client, &args.id).await?;
    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.update",
            &serde_json::json!({
                "id": resolved_id,
                "nextFollowUpAt": args.next_follow_up,
                "linkedinUrl": args.linkedin_url,
                "notes": args.notes,
                "assignedTo": args.assigned_to,
                "tier": args.tier,
                "contractValue": args.contract_value,
                "contractCurrency": args.contract_currency,
            }),
        )
        .await?;

    if crate::is_agent() {
        println!("{}", serde_json::json!({ "ok": true, "id": args.id }));
    } else {
        println!("Sponsor {} updated successfully.", args.id);
    }
    Ok(())
}

pub async fn update_contacts(args: UpdateContactsArgs) -> Result<()> {
    let client = require_client()?;

    let sponsor: serde_json::Value = client
        .query(
            "sponsor.crm.getById",
            Some(&serde_json::json!({ "id": args.id })),
        )
        .await?;

    let mut contact_persons = sponsor
        .get("contactPersons")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // Unmark existing primary contacts if this is a new primary
    if !args.secondary {
        for contact in &mut contact_persons {
            if let Some(obj) = contact.as_object_mut() {
                obj.insert("isPrimary".to_string(), serde_json::json!(false));
            }
        }
    }

    contact_persons.push(serde_json::json!({
        "_key": uuid::Uuid::new_v4().to_string(),
        "name": args.name,
        "email": args.email,
        "isPrimary": !args.secondary,
    }));

    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.update",
            &serde_json::json!({
                "id": args.id,
                "contactPersons": contact_persons,
            }),
        )
        .await?;

    if crate::is_agent() {
        println!("{}", serde_json::json!({ "ok": true, "id": args.id }));
    } else {
        println!("Contacts for sponsor {} updated successfully.", args.id);
    }
    Ok(())
}

pub async fn fetch_organizers(client: &TrpcClient) -> Result<Vec<crate::types::SpeakerRef>> {
    let organizers: Vec<crate::types::SpeakerRef> =
        client.query("sponsor.crm.listOrganizers", None).await?;
    Ok(organizers)
}

pub async fn assign(id: &str, speaker_id: Option<&str>) -> Result<()> {
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.update",
            &serde_json::json!({
                "id": id,
                "assignedTo": speaker_id,
            }),
        )
        .await?;

    if crate::is_agent() {
        println!(
            "{}",
            serde_json::json!({ "ok": true, "id": id, "speaker_id": speaker_id })
        );
    } else {
        match speaker_id {
            Some(sid) => println!("Sponsor {id} assigned to speaker {sid}."),
            None => println!("Sponsor {id} unassigned."),
        }
    }
    Ok(())
}

pub async fn history(id: &str, json: bool) -> Result<()> {
    let client = require_client()?;
    let mut sponsor = fetch_one(&client, id).await?;
    sponsor.activities = fetch_activities(&client, id).await?;

    if json || crate::is_agent() {
        if crate::is_agent() {
            println!("{}", serde_json::to_string(&sponsor.activities)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&sponsor.activities)?);
        }
    } else {
        display::print_sponsor_history(&sponsor);
    }
    Ok(())
}

pub async fn add_note(args: NoteArgs) -> Result<()> {
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.activities.create",
            &serde_json::json!({
                "sponsorForConferenceId": args.id,
                "activityType": args.kind,
                "description": args.description,
            }),
        )
        .await?;

    if crate::is_agent() {
        println!("{}", serde_json::json!({ "ok": true, "id": args.id }));
    } else {
        println!("Activity logged successfully.");
    }
    Ok(())
}

pub async fn move_stage(id: &str, stage: crate::types::SponsorStatus) -> Result<()> {
    let client = require_client()?;
    let resolved_id = resolve_id(&client, id).await?;
    let id = &resolved_id;
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.moveStage",
            &serde_json::json!({
                "id": id,
                "newStatus": stage,
            }),
        )
        .await?;

    if crate::is_agent() {
        println!(
            "{}",
            serde_json::json!({ "ok": true, "id": id, "stage": stage })
        );
    } else {
        println!("Sponsor moved to stage {stage}.");
    }
    Ok(())
}

pub async fn update_invoice(id: &str, status: &str) -> Result<()> {
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.updateInvoiceStatus",
            &serde_json::json!({
                "id": id,
                "status": status,
            }),
        )
        .await?;

    if crate::is_agent() {
        println!(
            "{}",
            serde_json::json!({ "ok": true, "id": id, "status": status })
        );
    } else {
        println!("Invoice status updated to {status}.");
    }
    Ok(())
}

pub async fn update_contract(id: &str, status: &str) -> Result<()> {
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.updateContractStatus",
            &serde_json::json!({
                "id": id,
                "status": status,
            }),
        )
        .await?;

    if crate::is_agent() {
        println!(
            "{}",
            serde_json::json!({ "ok": true, "id": id, "status": status })
        );
    } else {
        println!("Contract status updated to {status}.");
    }
    Ok(())
}

pub async fn send_registration(id: &str) -> Result<()> {
    let client = require_client()?;
    let resolved_id = resolve_id(&client, id).await?;

    let sp = crate::ui::spinner("Sending registration invite...");
    client
        .mutate::<serde_json::Value>(
            "registration.sendPortalInvite",
            &serde_json::json!({
                "sponsorForConferenceId": resolved_id,
            }),
        )
        .await?;

    sp.finish_and_clear();

    if crate::is_agent() {
        println!("{}", serde_json::json!({ "ok": true, "id": id }));
    } else {
        println!("Registration invite sent successfully.");
    }
    Ok(())
}
pub async fn generate_contract(id: &str, template: Option<&str>) -> Result<()> {
    let client = require_client()?;
    let resolved_id = resolve_id(&client, id).await?;

    // We need the sponsor to get the tier for findBest
    let sponsor = fetch_one(&client, &resolved_id).await?;
    let tier_id = sponsor.tier.as_ref().map(|t| t.id.clone());

    let template_id = if let Some(t) = template {
        t.to_string()
    } else {
        let find_best_input = serde_json::json!({
            "tierId": tier_id,
        });
        let best_template: serde_json::Value = client
            .query("sponsor.contractTemplates.findBest", Some(&find_best_input))
            .await?;
        if best_template.is_null() {
            anyhow::bail!("No suitable contract template found. Please specify one manually.");
        }
        best_template["_id"].as_str().unwrap().to_string()
    };

    let sp = crate::ui::spinner("Generating contract PDF...");
    let response: serde_json::Value = client
        .mutate(
            "sponsor.crm.generatePdf",
            &serde_json::json!({
                "sponsorForConferenceId": resolved_id,
                "templateId": template_id,
            }),
        )
        .await?;

    sp.finish_and_clear();

    if crate::is_agent() {
        println!("{}", serde_json::json!({ "ok": true, "id": id }));
    } else {
        println!("Contract generated successfully.");
        if let Some(base64) = response["pdf"].as_str() {
            println!("(Base64 data length: {})", base64.len());
        }
    }
    Ok(())
}

pub async fn send_contract(id: &str, template: Option<&str>) -> Result<()> {
    let client = require_client()?;
    let resolved_id = resolve_id(&client, id).await?;

    // We need the sponsor to get the tier for findBest
    let sponsor = fetch_one(&client, &resolved_id).await?;
    let tier_id = sponsor.tier.as_ref().map(|t| t.id.clone());

    let template_id = if let Some(t) = template {
        t.to_string()
    } else {
        let find_best_input = serde_json::json!({
            "tierId": tier_id,
        });
        let best_template: serde_json::Value = client
            .query("sponsor.contractTemplates.findBest", Some(&find_best_input))
            .await?;
        if best_template.is_null() {
            anyhow::bail!("No suitable contract template found. Please specify one manually.");
        }
        best_template["_id"].as_str().unwrap().to_string()
    };

    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.sendContract",
            &serde_json::json!({
                "sponsorForConferenceId": resolved_id,
                "templateId": template_id,
            }),
        )
        .await?;

    if crate::is_agent() {
        println!("{}", serde_json::json!({ "ok": true, "id": id }));
    } else {
        println!("Contract generated and sent successfully.");
    }
    Ok(())
}

pub async fn delete_activity(id: &str, yes: bool) -> Result<()> {
    if !yes {
        if !console::Term::stdout().is_term() {
            anyhow::bail!("Confirmation required in non-interactive mode. Pass -y to confirm.");
        }
        let confirmed = dialoguer::Confirm::new()
            .with_prompt(format!("Are you sure you want to delete activity {id}?"))
            .default(false)
            .interact()?;

        if !confirmed {
            anyhow::bail!("Deletion cancelled.");
        }
    }

    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.activities.delete",
            &serde_json::json!({ "id": id }),
        )
        .await?;

    if crate::is_agent() {
        println!("{}", serde_json::json!({ "ok": true, "id": id }));
    } else {
        println!("Activity deleted successfully.");
    }
    Ok(())
}

pub async fn signature_status(id: &str) -> Result<()> {
    let client = require_client()?;
    let res: serde_json::Value = client
        .mutate(
            "sponsor.crm.checkSignatureStatus",
            &serde_json::json!({ "id": id }),
        )
        .await?;

    let status = res
        .get("contractStatus")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown");

    if crate::is_agent() {
        println!(
            "{}",
            serde_json::json!({ "ok": true, "id": id, "status": status })
        );
    } else {
        println!("Signature status synced. Current contract status: {status}");
    }
    Ok(())
}

pub async fn sync_audience() -> Result<()> {
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>("sponsor.crm.syncAudience", &serde_json::json!({}))
        .await?;

    if crate::is_agent() {
        println!("{}", serde_json::json!({ "ok": true }));
    } else {
        println!("Sponsor email audience synced successfully.");
    }
    Ok(())
}

pub async fn create(args: CreateArgs) -> Result<()> {
    let client = require_client()?;
    let config = crate::config::load()?;

    // 1. Create the base sponsor
    let sponsor: serde_json::Value = client
        .mutate(
            "sponsor.create",
            &serde_json::json!({
                "name": args.name,
                "website": args.website,
            }),
        )
        .await?;

    let sponsor_id = sponsor
        .get("id")
        .or_else(|| sponsor.get("_id"))
        .and_then(|id| id.as_str())
        .context(format!("Missing sponsor ID in response: {sponsor:?}"))?;

    // 2. Link to conference (CRM)
    let mut contact_persons = serde_json::json!([]);
    if let Some(name) = args.contact_name {
        contact_persons = serde_json::json!([{
            "_key": uuid::Uuid::new_v4().to_string(),
            "name": name,
            "email": args.contact_email,
            "isPrimary": true,
        }]);
    }

    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.create",
            &serde_json::json!({
                "sponsor": sponsor_id,
                "conference": config.conference_id,
                "status": args.status,
                "contractStatus": "none",
                "invoiceStatus": "not-sent",
                "notes": args.notes,
                "contactPersons": contact_persons,
            }),
        )
        .await?;

    if crate::is_agent() {
        println!("{}", serde_json::json!({ "ok": true, "id": sponsor_id }));
    } else {
        println!("Sponsor '{}' added to CRM as {}.", args.name, args.status);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SponsorForConference, SponsorRef};

    fn make_sponsor(id: &str, name: &str) -> SponsorForConference {
        SponsorForConference {
            id: id.to_string(),
            status: crate::types::SponsorStatus::Prospect,
            contract_status: Some("none".to_string()),
            invoice_status: Some("not-sent".to_string()),
            sponsor: Some(SponsorRef {
                id: "s1".to_string(),
                name: name.to_string(),
                website: None,
                linkedin_url: None,
            }),
            tier: None,
            assigned_to: None,
            contact_persons: vec![],
            billing: None,
            contract_value: None,
            contract_currency: Some("NOK".to_string()),
            notes: None,
            tags: vec![],
            contract_signed_at: None,
            invoice_sent_at: None,
            invoice_paid_at: None,
            activities: vec![],
            last_activity: None,
            activity_count: Some(0),
            next_follow_up_at: None,
            outreach_count: None,
            contact_initiated_at: None,
        }
    }

    #[test]
    fn test_find_sponsor_by_name() {
        let sponsors = vec![
            make_sponsor("1", "Acme Corp"),
            make_sponsor("2", "Global Tech"),
            make_sponsor("3", "global tech"),
        ];

        // Exact match
        let matches = find_sponsor_by_name(&sponsors, "Acme Corp").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "1");

        // Case insensitive match
        let matches = find_sponsor_by_name(&sponsors, "acme CORP").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "1");

        // Multiple matches
        let matches = find_sponsor_by_name(&sponsors, "Global Tech").unwrap();
        assert_eq!(matches.len(), 2);

        // No match
        let matches = find_sponsor_by_name(&sponsors, "Unknown");
        assert!(matches.is_none());
    }
}
