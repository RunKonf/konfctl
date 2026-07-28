use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::null_to_vec;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Speaker {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub slug: serde_json::Value,
    #[serde(default)]
    pub email: serde_json::Value,
    #[serde(default)]
    pub title: serde_json::Value,
    #[serde(default)]
    pub company: serde_json::Value,
    #[serde(default)]
    pub image: serde_json::Value,
    #[serde(default)]
    pub bio: serde_json::Value,
    #[serde(default, deserialize_with = "null_to_vec")]
    pub links: Vec<String>,
    #[serde(default, deserialize_with = "null_to_vec")]
    pub flags: Vec<SpeakerFlag>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum SpeakerFlag {
    Local,
    FirstTime,
    Diverse,
    RequiresFunding,
    Keynote,
    Hidden,
    Internal,
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for SpeakerFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            Self::Local => "local",
            Self::FirstTime => "first-time",
            Self::Diverse => "diverse",
            Self::RequiresFunding => "requires-funding",
            Self::Keynote => "keynote",
            Self::Hidden => "hidden",
            Self::Internal => "internal",
            Self::Unknown => "unknown",
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerSummary {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerRef {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerCreateInput {
    pub name: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<SpeakerFlag>>,
}
