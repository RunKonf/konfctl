use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use colored::Colorize;
use konfctl::commands;

#[derive(Parser)]
#[command(
    name = "konf",
    about = "Konf CLI — run your conference from the terminal. Optimized for humans and LLM agents.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Enable agent-optimized output and hints
    #[arg(long, global = true)]
    agent: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Authenticate via browser and select a conference
    Login {
        /// Optional custom conference API URL to authenticate against
        url: Option<String>,
    },
    /// Clear stored credentials
    Logout {
        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Show current authentication and conference context
    Status,
    /// Get machine-readable context and capability map for LLM agents
    AgentInfo {
        /// Output as JSON
        #[arg(long, default_value_t = true)]
        json: bool,
    },
    /// Output full CLI specification as JSON for agent ingestion
    HelpJson,
    /// Organizer administration commands
    #[command(subcommand)]
    Admin(AdminCommand),
    /// Manage conference-specific agent instructions
    Agents(commands::agents::AgentArgs),
}

#[derive(Subcommand)]
enum AdminCommand {
    /// Manage talk proposals
    #[command(subcommand)]
    Proposals(ProposalCommand),
    /// Manage sponsor pipeline
    #[command(subcommand)]
    Sponsors(SponsorCommand),
    /// Manage speaker profiles
    #[command(subcommand)]
    Speakers(commands::speakers::SpeakerCommand),
    /// Manage featured content on the front page
    Featured(commands::featured::FeaturedArgs),
    /// Organizer inbox and messaging
    Messages(commands::messages::MessageArgs),
    /// Manage schedules
    #[command(subcommand)]
    Schedule(commands::schedule::ScheduleCommand),
    /// Show conference status summary (sponsors, proposals, tickets, targets)
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum ProposalCommand {
    /// List all proposals (interactive by default, or use flags for scripting)
    List(commands::proposals::ListArgs),
    /// Add a new manual proposal
    Add(commands::proposals::CreateArgs),
    /// Show proposal details
    Get {
        /// Proposal ID
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Submit or update a review for a proposal
    Review(commands::proposals::ReviewArgs),
    /// Delete a proposal
    Delete(commands::proposals::DeleteArgs),
    /// Perform an action on a proposal (accept, reject, etc.)
    Action(commands::proposals::ActionArgs),
    /// Update proposal details
    Update(commands::proposals::UpdateArgs),
    /// Find the next unreviewed proposal
    NextReview,
    /// Add a speaker to an existing proposal
    AddSpeaker {
        /// Proposal ID
        proposal_id: String,
        /// Speaker ID or Email
        speaker: String,
    },
}

#[derive(Subcommand)]
enum SponsorCommand {
    /// List sponsor pipeline (interactive by default, or use flags for scripting)
    List(commands::sponsors::ListArgs),
    /// Add a new sponsor to the CRM
    Add(commands::sponsors::CreateArgs),
    /// Update sponsor details
    Update(commands::sponsors::UpdateArgs),
    /// Show sponsor details
    Get {
        /// Sponsor-for-conference ID
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Bulk overwrite contact persons (replaces all existing)
    UpdateContacts(commands::sponsors::UpdateContactsArgs),
    /// Show sponsor history (activities, notes, stage changes)
    History {
        /// Sponsor-for-conference ID
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Add a manual note/activity to a sponsor's history
    Note(commands::sponsors::NoteArgs),
    /// Send an email to a sponsor using templates
    Email(commands::sponsors::EmailArgs),
    /// Move a sponsor to a different pipeline stage
    MoveStage {
        /// Sponsor-for-conference ID
        id: String,
        /// New stage
        #[arg(value_enum)]
        stage: konfctl::types::SponsorStatus,
    },
    /// Update invoice status
    UpdateInvoice {
        /// Sponsor-for-conference ID
        id: String,
        /// New invoice status
        status: String,
    },
    /// Update contract status
    UpdateContract {
        /// Sponsor-for-conference ID
        id: String,
        /// New contract status
        status: String,
    },
    /// Generate and send contract for signing
    SendContract {
        /// Sponsor-for-conference ID
        id: String,
        /// Template slug (optional)
        #[arg(long)]
        template: Option<String>,
    },
    /// Poll and sync signature status from provider
    SignatureStatus {
        /// Sponsor-for-conference ID
        id: String,
    },
    /// Delete a custom activity entry (note, email, call, meeting)
    DeleteActivity {
        /// Activity ID
        id: String,
        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Assign an organizer to a sponsor
    Assign {
        /// Sponsor-for-conference ID
        id: String,
        /// Speaker ID
        #[arg(long)]
        speaker_id: Option<String>,
    },
    /// List all conference organizers
    Organizers,
    /// Sync confirmed sponsors with newsletter audience
    SyncAudience,
}

async fn run_command(command: Command, is_agent: bool) -> Result<()> {
    match command {
        Command::Login { url } => commands::login::run(url),
        Command::Logout { yes } => commands::logout::run(yes),
        Command::Status => commands::status::run(),
        Command::AgentInfo { json } => commands::agent_discovery::run_agent_info(json).await,
        Command::HelpJson => commands::agent_discovery::run_help_json(&Cli::command()),
        Command::Admin(cmd) => run_admin_command(cmd, is_agent).await,
        Command::Agents(args) => commands::agents::run(args).await,
    }
}

fn check_agent_guard(is_agent: bool, command_hint: &str) -> Result<()> {
    if is_agent {
        anyhow::bail!(
            "AGENT_GUARD_BLOCK: This command alters production data or sends external communications and cannot be executed autonomously. You must stop and ask the human user to run the following command in their terminal:\n\nkonf {command_hint}\n\nDo not attempt to bypass this."
        );
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn run_admin_command(cmd: AdminCommand, is_agent: bool) -> Result<()> {
    match cmd {
        AdminCommand::Proposals(cmd) => match cmd {
            ProposalCommand::List(args) => commands::proposals::list(args).await,
            ProposalCommand::Add(args) => commands::proposals::add(args).await,
            ProposalCommand::Get { id, json } => commands::proposals::get(&id, json).await,
            ProposalCommand::Review(args) => commands::proposals::review(args).await,
            ProposalCommand::Delete(args) => {
                check_agent_guard(is_agent, &format!("admin proposals delete {}", args.id))?;
                commands::proposals::delete(args).await
            }
            ProposalCommand::Action(args) => {
                check_agent_guard(
                    is_agent,
                    &format!("admin proposals action {} {:?}", args.id, args.action),
                )?;
                commands::proposals::action(args).await
            }
            ProposalCommand::Update(args) => commands::proposals::update(args).await,
            ProposalCommand::NextReview => commands::proposals::next_review().await,
            ProposalCommand::AddSpeaker {
                proposal_id,
                speaker,
            } => commands::proposals::add_speaker(&proposal_id, &speaker).await,
        },
        AdminCommand::Sponsors(cmd) => match cmd {
            SponsorCommand::List(args) => commands::sponsors::list(args).await,
            SponsorCommand::Add(args) => commands::sponsors::create(args).await,
            SponsorCommand::Update(args) => {
                check_agent_guard(is_agent, &format!("admin sponsors update {}", args.id))?;
                commands::sponsors::update(args).await
            }
            SponsorCommand::UpdateContacts(args) => {
                check_agent_guard(
                    is_agent,
                    &format!("admin sponsors update-contacts {}", args.id),
                )?;
                commands::sponsors::update_contacts(args).await
            }
            SponsorCommand::Get { id, json } => commands::sponsors::get(&id, json).await,
            SponsorCommand::History { id, json } => commands::sponsors::history(&id, json).await,
            SponsorCommand::Note(args) => commands::sponsors::add_note(args).await,
            SponsorCommand::Email(args) => {
                check_agent_guard(is_agent, &format!("admin sponsors email {}", args.id))?;
                commands::sponsors::email::run(args).await
            }
            SponsorCommand::MoveStage { id, stage } => {
                commands::sponsors::move_stage(&id, stage).await
            }
            SponsorCommand::UpdateInvoice { id, status } => {
                commands::sponsors::update_invoice(&id, &status).await
            }
            SponsorCommand::UpdateContract { id, status } => {
                commands::sponsors::update_contract(&id, &status).await
            }
            SponsorCommand::SendContract { id, template } => {
                check_agent_guard(is_agent, &format!("admin sponsors send-contract {id}"))?;
                commands::sponsors::send_contract(&id, template.as_deref()).await
            }
            SponsorCommand::SignatureStatus { id } => {
                commands::sponsors::signature_status(&id).await
            }
            SponsorCommand::DeleteActivity { id, yes } => {
                check_agent_guard(is_agent, &format!("admin sponsors delete-activity {id}"))?;
                commands::sponsors::delete_activity(&id, yes).await
            }
            SponsorCommand::Assign { id, speaker_id } => {
                commands::sponsors::assign(&id, speaker_id.as_deref()).await
            }
            SponsorCommand::Organizers => {
                let client = commands::require_client()?;
                let orgs = commands::sponsors::fetch_organizers(&client).await?;
                println!("{}", serde_json::to_string_pretty(&orgs)?);
                Ok(())
            }
            SponsorCommand::SyncAudience => commands::sponsors::sync_audience().await,
        },
        AdminCommand::Speakers(cmd) => match cmd {
            commands::speakers::SpeakerCommand::List(args) => commands::speakers::list(args).await,
            commands::speakers::SpeakerCommand::Get { id, json } => {
                commands::speakers::get(&id, json).await
            }
            commands::speakers::SpeakerCommand::Add(args) => commands::speakers::add(args).await,
            commands::speakers::SpeakerCommand::Delete { id, yes } => {
                check_agent_guard(is_agent, &format!("admin speakers delete {id}"))?;
                commands::speakers::delete(&id, yes).await
            }
            commands::speakers::SpeakerCommand::Broadcast {
                subject,
                message,
                sync,
            } => {
                check_agent_guard(is_agent, "admin speakers broadcast")?;
                commands::speakers::broadcast(subject.as_deref(), message.as_deref(), sync).await
            }
            commands::speakers::SpeakerCommand::FindOrCreate(args) => {
                commands::speakers::find_or_create(args).await
            }
            commands::speakers::SpeakerCommand::SyncAudience => {
                commands::speakers::sync_audience().await
            }
        },
        AdminCommand::Featured(args) => commands::featured::run(args).await,
        AdminCommand::Messages(args) => {
            use commands::messages::MessageCommand;
            match args.command {
                MessageCommand::List(list_args) => commands::messages::list(list_args).await,
                MessageCommand::Get { id, json } => commands::messages::get(&id, json).await,
                MessageCommand::Reply { id, message } => {
                    check_agent_guard(is_agent, &format!("admin messages reply {}", id))?;
                    commands::messages::reply(&id, &message).await
                }
                MessageCommand::New {
                    speaker,
                    subject,
                    message,
                } => {
                    check_agent_guard(
                        is_agent,
                        &format!("admin messages new --speaker {}", speaker),
                    )?;
                    commands::messages::start_new(&speaker, &subject, &message).await
                }
                MessageCommand::Status { id, status, yes } => {
                    if is_agent && !yes {
                        anyhow::bail!("Agent must pass -y/--yes to confirm mutating thread status.");
                    }
                    commands::messages::set_status(&id, status).await
                }
                MessageCommand::Assign { id, to, yes } => {
                    if is_agent && !yes {
                        anyhow::bail!("Agent must pass -y/--yes to confirm reassigning thread.");
                    }
                    commands::messages::set_assignee(&id, to.as_deref()).await
                }
                MessageCommand::Archive { id, unarchive, yes } => {
                    if is_agent && !yes {
                        anyhow::bail!("Agent must pass -y/--yes to confirm archiving thread.");
                    }
                    commands::messages::set_archive(&id, unarchive).await
                }
            }
        }
        AdminCommand::Schedule(cmd) => match cmd {
            commands::schedule::ScheduleCommand::List(args) => commands::schedule::list(args).await,
            commands::schedule::ScheduleCommand::Get { id, json } => {
                commands::schedule::get(&id, json).await
            }
            commands::schedule::ScheduleCommand::Promote { id } => {
                check_agent_guard(is_agent, &format!("admin schedule promote {id}"))?;
                commands::schedule::promote(&id).await
            }
            commands::schedule::ScheduleCommand::Delete { id, yes } => {
                check_agent_guard(is_agent, &format!("admin schedule delete {id}"))?;
                commands::schedule::delete(&id, yes).await
            }
            commands::schedule::ScheduleCommand::Save { payload } => {
                check_agent_guard(is_agent, "admin schedule save")?;
                commands::schedule::save(&payload).await
            }
        },
        AdminCommand::Status { json } => commands::admin_status::run(json).await,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let is_agent = cli.agent;
    konfctl::set_is_agent(is_agent);

    let res = run_command(cli.command, is_agent).await;

    if let Err(e) = res {
        if is_agent {
            let mut hints = Vec::new();
            let error_str = e.to_string();
            let mut error_code = "UNKNOWN_ERROR";

            if error_str.contains("Authentication required") || error_str.contains("unauthorized") {
                error_code = "AUTH_REQUIRED";
                hints.push("Run 'konf login' to authenticate.");
            } else if error_str.contains("not found") {
                error_code = "NOT_FOUND";
                hints.push("Use 'list' commands with '--search' or '--all' to verify IDs.");
            } else if error_str.contains("conference context") {
                error_code = "CONFERENCE_NOT_SET";
                hints.push("Run 'konf status' to verify your active conference.");
            } else if error_str.contains("AGENT_GUARD_BLOCK") {
                error_code = "AGENT_GUARD_BLOCK";
            }

            let err_json = serde_json::json!({
                "error_code": error_code,
                "error": error_str,
                "hints": hints
            });
            eprintln!("{}", serde_json::to_string(&err_json)?);
        } else {
            eprintln!("{} {}", "Error:".red().bold(), e);
        }
        std::process::exit(1);
    }

    Ok(())
}
