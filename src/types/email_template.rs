use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TemplateCategory {
    ColdOutreach,
    ReturningSponsor,
    International,
    LocalCommunity,
    FollowUp,
    Contract,
    Custom,
    #[default]
    #[serde(other)]
    Unknown,
}

impl<'de> Deserialize<'de> for TemplateCategory {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        enum Helper {
            ColdOutreach,
            ReturningSponsor,
            International,
            LocalCommunity,
            FollowUp,
            Contract,
            Custom,
            #[serde(other)]
            Unknown,
        }
        Ok(match Helper::deserialize(deserializer)? {
            Helper::ColdOutreach => Self::ColdOutreach,
            Helper::ReturningSponsor => Self::ReturningSponsor,
            Helper::International => Self::International,
            Helper::LocalCommunity => Self::LocalCommunity,
            Helper::FollowUp => Self::FollowUp,
            Helper::Contract => Self::Contract,
            Helper::Custom => Self::Custom,
            Helper::Unknown => Self::Unknown,
        })
    }
}

impl fmt::Display for TemplateCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            Self::ColdOutreach => "cold-outreach",
            Self::ReturningSponsor => "returning-sponsor",
            Self::International => "international",
            Self::LocalCommunity => "local-community",
            Self::FollowUp => "follow-up",
            Self::Contract => "contract",
            Self::Custom => "custom",
            Self::Unknown => "unknown",
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TemplateLanguage {
    #[serde(rename = "no")]
    Norwegian,
    #[serde(rename = "en")]
    English,
    #[default]
    #[serde(other)]
    Unknown,
}

impl<'de> Deserialize<'de> for TemplateLanguage {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        enum Helper {
            #[serde(rename = "no")]
            Norwegian,
            #[serde(rename = "en")]
            English,
            #[serde(other)]
            Unknown,
        }
        Ok(match Helper::deserialize(deserializer)? {
            Helper::Norwegian => Self::Norwegian,
            Helper::English => Self::English,
            Helper::Unknown => Self::Unknown,
        })
    }
}

impl fmt::Display for TemplateLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            Self::Norwegian => "🇳🇴 Norwegian",
            Self::English => "🇬🇧 English",
            Self::Unknown => "unknown",
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSlug {
    pub current: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SponsorEmailTemplate {
    #[serde(rename = "_id")]
    pub id: String,
    pub title: String,
    pub slug: TemplateSlug,
    pub category: TemplateCategory,
    pub language: TemplateLanguage,
    pub subject: String,
    #[serde(default)]
    pub body_markdown: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_default: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailRecipient {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateListResponse {
    pub templates: Vec<SponsorEmailTemplate>,
    pub variables: HashMap<String, String>,
    pub recipients: Vec<EmailRecipient>,
    #[serde(default)]
    pub sponsor_name: Option<String>,
    #[serde(default)]
    pub suggested_category: TemplateCategory,
    #[serde(default)]
    pub suggested_language: TemplateLanguage,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendEmailResponse {
    pub success: bool,
    #[serde(default)]
    pub email_id: Option<String>,
    #[serde(default)]
    pub recipient_count: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_full_template() {
        let json = serde_json::json!({
            "_id": "tpl-1",
            "title": "Cold Outreach Norwegian",
            "slug": { "current": "cold-outreach-no" },
            "category": "cold-outreach",
            "language": "no",
            "subject": "Sponsor {{{CONFERENCE_TITLE}}}?",
            "bodyMarkdown": "Dear **{{{CONTACT_NAMES}}}**,\n\nWe invite you...",
            "description": "Standard Norwegian outreach template",
            "isDefault": true,
            "sortOrder": 1
        });

        let t: SponsorEmailTemplate = serde_json::from_value(json).unwrap();
        assert_eq!(t.id, "tpl-1");
        assert_eq!(t.title, "Cold Outreach Norwegian");
        assert_eq!(t.slug.current, "cold-outreach-no");
        assert_eq!(t.category, TemplateCategory::ColdOutreach);
        assert_eq!(t.language, TemplateLanguage::Norwegian);
        assert!(t.body_markdown.unwrap().contains("{{{CONTACT_NAMES}}}"));
        assert_eq!(t.is_default, Some(true));
        assert_eq!(t.sort_order, Some(1));
    }

    #[test]
    fn deserialize_minimal_template() {
        let json = serde_json::json!({
            "_id": "tpl-2",
            "title": "Follow-up",
            "slug": { "current": "follow-up-en" },
            "category": "follow-up",
            "language": "en",
            "subject": "Following up"
        });

        let t: SponsorEmailTemplate = serde_json::from_value(json).unwrap();
        assert_eq!(t.id, "tpl-2");
        assert_eq!(t.category, TemplateCategory::FollowUp);
        assert_eq!(t.language, TemplateLanguage::English);
        assert!(t.body_markdown.is_none());
        assert!(t.description.is_none());
        assert!(t.is_default.is_none());
    }

    #[test]
    fn unknown_category_deserializes_gracefully() {
        let json = serde_json::json!({
            "_id": "tpl-3",
            "title": "Future",
            "slug": { "current": "future" },
            "category": "some-new-category",
            "language": "en",
            "subject": "Test"
        });

        let t: SponsorEmailTemplate = serde_json::from_value(json).unwrap();
        assert_eq!(t.category, TemplateCategory::Unknown);
    }

    #[test]
    fn unknown_language_deserializes_gracefully() {
        let json = serde_json::json!({
            "_id": "tpl-4",
            "title": "Swedish",
            "slug": { "current": "se" },
            "category": "custom",
            "language": "se",
            "subject": "Hej"
        });

        let t: SponsorEmailTemplate = serde_json::from_value(json).unwrap();
        assert_eq!(t.language, TemplateLanguage::Unknown);
    }

    #[test]
    fn deserialize_template_list_response() {
        let json = serde_json::json!({
            "templates": [{
                "_id": "tpl-1",
                "title": "Cold Outreach",
                "slug": { "current": "cold-outreach-no" },
                "category": "cold-outreach",
                "language": "no",
                "subject": "Sponsor us?",
                "bodyMarkdown": "Hello **world**"
            }],
            "variables": {
                "SPONSOR_NAME": "Acme Corp",
                "CONFERENCE_TITLE": "Cloud Native Days 2026"
            },
            "recipients": [
                { "name": "Jane", "email": "jane@acme.com" }
            ],
            "sponsorName": "Acme Corp",
            "suggestedCategory": "cold-outreach",
            "suggestedLanguage": "no"
        });

        let resp: TemplateListResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.templates.len(), 1);
        assert_eq!(resp.variables.get("SPONSOR_NAME").unwrap(), "Acme Corp");
        assert_eq!(resp.sponsor_name.as_deref(), Some("Acme Corp"));
        assert_eq!(resp.recipients.len(), 1);
        assert_eq!(resp.recipients[0].email, "jane@acme.com");
        assert_eq!(resp.suggested_category, TemplateCategory::ColdOutreach);
        assert_eq!(resp.suggested_language, TemplateLanguage::Norwegian);
    }

    #[test]
    fn deserialize_send_email_response() {
        let json = serde_json::json!({
            "success": true,
            "emailId": "email-abc123",
            "recipientCount": 2
        });

        let resp: SendEmailResponse = serde_json::from_value(json).unwrap();
        assert!(resp.success);
        assert_eq!(resp.email_id.as_deref(), Some("email-abc123"));
        assert_eq!(resp.recipient_count, Some(2));
    }

    #[test]
    fn category_display() {
        assert_eq!(TemplateCategory::ColdOutreach.to_string(), "cold-outreach");
        assert_eq!(
            TemplateCategory::ReturningSponsor.to_string(),
            "returning-sponsor"
        );
        assert_eq!(TemplateCategory::FollowUp.to_string(), "follow-up");
    }

    #[test]
    fn language_display() {
        assert!(
            TemplateLanguage::Norwegian
                .to_string()
                .contains("Norwegian")
        );
        assert!(TemplateLanguage::English.to_string().contains("English"));
    }
}
