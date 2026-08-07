use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ScheduleStatus {
    Draft,
    Official,
    Archived,
}

impl fmt::Display for ScheduleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScheduleStatus::Draft => write!(f, "draft"),
            ScheduleStatus::Official => write!(f, "official"),
            ScheduleStatus::Archived => write!(f, "archived"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_rev")]
    pub rev: Option<String>,
    pub date: String,
    pub status: Option<ScheduleStatus>,
    pub version: Option<i32>,
    pub tracks: Option<Vec<serde_json::Value>>,
    pub conference: Option<serde_json::Value>,
    pub owner: Option<serde_json::Value>,
}
