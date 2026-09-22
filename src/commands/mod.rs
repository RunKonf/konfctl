use anyhow::{Context, Result};

use crate::client::TrpcClient;
use crate::config;

pub mod admin_status;
pub mod admin_tickets;
pub mod agent_discovery;
pub mod agents;
pub mod featured;
pub mod login;
pub mod logout;
pub mod messages;
pub mod proposals;
pub mod schedule;
pub mod speakers;
pub mod sponsors;
pub mod status;

pub fn require_client() -> Result<TrpcClient> {
    let cfg = config::load().context("Not logged in. Run `konf login` first.")?;
    Ok(TrpcClient::from_config(&cfg))
}

pub fn read_string_or_file(val: String) -> Result<String> {
    if let Some(path) = val.strip_prefix('@') {
        std::fs::read_to_string(path).with_context(|| format!("Failed to read file: {path}"))
    } else if val == "-" {
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)?;
        Ok(buf)
    } else {
        Ok(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_read_string_or_file_literal() {
        let input = "just a normal string".to_string();
        let result = read_string_or_file(input).unwrap();
        assert_eq!(result, "just a normal string");
    }

    #[test]
    fn test_read_string_or_file_from_file() {
        // Create a temporary file
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, "file content here").unwrap();

        let path = temp_file.path().to_str().unwrap();
        let input = format!("@{path}");

        let result = read_string_or_file(input).unwrap();
        assert_eq!(result.trim(), "file content here");
    }

    #[test]
    fn test_read_string_or_file_missing_file() {
        let input = "@this_file_does_not_exist_at_all.txt".to_string();
        let result = read_string_or_file(input);
        assert!(result.is_err());
    }
}
