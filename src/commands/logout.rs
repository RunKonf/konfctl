use anyhow::Result;

use crate::config;

pub fn run(yes: bool) -> Result<()> {
    if !yes && console::Term::stdout().is_term() {
        let confirmed = dialoguer::Confirm::new()
            .with_prompt("Are you sure you want to log out?")
            .default(false)
            .interact()?;

        if !confirmed {
            anyhow::bail!("Logout cancelled.");
        }
    }

    if config::delete()? {
        println!("Logged out.");
    } else {
        println!("Not logged in.");
    }
    Ok(())
}
