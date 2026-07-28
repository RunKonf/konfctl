use anyhow::Result;

use crate::config;

pub fn run() -> Result<()> {
    if !config::exists() {
        println!("Not logged in. Run `konf login` to authenticate.");
        return Ok(());
    }

    let cfg = config::load()?;
    if let Some(name) = &cfg.name {
        println!("Logged in:  {name}");
    }
    println!("Conference: {}", cfg.conference_title);
    println!("API URL:    {}", cfg.api_url);
    Ok(())
}
