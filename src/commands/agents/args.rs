use clap::{Args, Subcommand};

#[derive(Args)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub command: AgentCommand,
}

#[derive(Subcommand)]
pub enum AgentCommand {
    /// Get conference-specific context and rules for agents
    Get {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Update agent context and rules
    Set(SetArgs),
}

#[derive(Args)]
pub struct SetArgs {
    /// Conference goals, mission, and scope
    #[arg(long)]
    pub context: Option<String>,

    /// Criteria for judging talk proposals
    #[arg(long)]
    pub review_config: Option<String>,

    /// Tone and behavioral rules for CRM
    #[arg(long)]
    pub crm_config: Option<String>,
}
