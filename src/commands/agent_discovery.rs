use anyhow::Result;
use clap::Command;
use serde_json::{Value, json};

use crate::commands::require_client;

pub async fn run_agent_info(json_out: bool) -> Result<()> {
    let client_res = require_client();
    let config = crate::config::load().ok();
    let agents_config: Option<crate::types::AgentConfig> = if let Ok(ref client) = client_res {
        client.query("agents.get", None).await.ok()
    } else {
        None
    };

    if json_out {
        let info = json!({
            "environment": {
                "conferenceId": config.as_ref().map(|c| c.conference_id.clone()),
                "authStatus": if client_res.is_ok() { "authenticated" } else { "unauthenticated" },
                "configPath": crate::config::config_path().ok().map(|p| p.to_string_lossy().to_string()),
            },
            "capabilities": {
                "macros": [
                    {
                        "name": "admin proposals add-speaker",
                        "description": "Atomic speaker assignment by ID or Email"
                    },
                    {
                        "name": "admin speakers find-or-create",
                        "description": "Idempotent speaker creation"
                    }
                ],
                "flags": {
                    "--compact": "Reduces list output tokens significantly (Sponsors/Speakers/Proposals)",
                    "--agent": "Globally enables token-optimized JSON output, machine-readable errors, and bypasses interactive prompts"
                }
            },
            "persona": agents_config
        });
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        println!("Run with --json for machine-readable output.");
    }

    Ok(())
}

pub fn run_help_json(cmd: &Command) -> Result<()> {
    let json = serialize_command(cmd);
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

fn serialize_command(cmd: &Command) -> Value {
    let subcommands: Vec<Value> = cmd.get_subcommands().map(serialize_command).collect();

    let args: Vec<Value> = cmd
        .get_arguments()
        .filter(|arg| {
            let id = arg.get_id().as_str();
            id != "help" && id != "version"
        })
        .map(|arg| {
            json!({
                "name": arg.get_id().to_string(),
                "long": arg.get_long(),
                "short": arg.get_short(),
                "help": arg.get_help().map(std::string::ToString::to_string),
                "required": arg.is_required_set(),
            })
        })
        .collect();

    json!({
        "name": cmd.get_name().to_string(),
        "about": cmd.get_about().map(std::string::ToString::to_string),
        "subcommands": if subcommands.is_empty() { Value::Null } else { json!(subcommands) },
        "arguments": if args.is_empty() { Value::Null } else { json!(args) },
    })
}
