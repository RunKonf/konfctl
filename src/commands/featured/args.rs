use clap::{Args, Subcommand};

#[derive(Args)]
pub struct FeaturedArgs {
    #[command(subcommand)]
    pub command: FeaturedCommand,
}

#[derive(Subcommand)]
pub enum FeaturedCommand {
    /// List all featured speakers and talks
    List {
        #[arg(long)]
        json: bool,
    },
    /// Add a featured speaker
    AddSpeaker {
        /// Speaker ID
        id: String,
    },
    /// Remove a featured speaker
    RemoveSpeaker {
        /// Speaker ID
        id: String,

        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Add a featured talk
    AddTalk {
        /// Proposal ID
        id: String,
    },
    /// Remove a featured talk
    RemoveTalk {
        /// Proposal ID
        id: String,

        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },
}
