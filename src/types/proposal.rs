use std::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use super::{Speaker, null_to_vec};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ValueEnum, Default)]
#[serde(rename_all = "camelCase")]
pub enum ProposalSortBy {
    #[default]
    Created,
    Title,
    Status,
    Speaker,
    Rating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ValueEnum, Default)]
#[serde(rename_all = "camelCase")]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ValueEnum, Default)]
#[serde(rename_all = "camelCase")]
pub enum ReviewStatus {
    #[default]
    All,
    Reviewed,
    Unreviewed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "camelCase")]
pub enum ProposalStatus {
    Submitted,
    Accepted,
    Confirmed,
    Waitlisted,
    Rejected,
    Withdrawn,
    Draft,
    Deleted,
    /// Catch-all for unknown values from the API (future-proofing).
    #[value(skip)]
    #[serde(other)]
    Unknown,
}

// Custom deserializer: try known variants, fall back to Unknown.
impl<'de> Deserialize<'de> for ProposalStatus {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        enum Helper {
            Submitted,
            Accepted,
            Confirmed,
            Waitlisted,
            Rejected,
            Withdrawn,
            Draft,
            Deleted,
            #[serde(other)]
            Unknown,
        }
        Ok(match Helper::deserialize(deserializer)? {
            Helper::Submitted => Self::Submitted,
            Helper::Accepted => Self::Accepted,
            Helper::Confirmed => Self::Confirmed,
            Helper::Waitlisted => Self::Waitlisted,
            Helper::Rejected => Self::Rejected,
            Helper::Withdrawn => Self::Withdrawn,
            Helper::Draft => Self::Draft,
            Helper::Deleted => Self::Deleted,
            Helper::Unknown => Self::Unknown,
        })
    }
}

impl fmt::Display for ProposalStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            Self::Submitted => "Submitted",
            Self::Accepted => "Accepted",
            Self::Confirmed => "Confirmed",
            Self::Waitlisted => "Waitlisted",
            Self::Rejected => "Rejected",
            Self::Withdrawn => "Withdrawn",
            Self::Draft => "Draft",
            Self::Deleted => "Deleted",
            Self::Unknown => "Unknown",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ValueEnum)]
pub enum ProposalFormat {
    #[serde(rename = "lightning_10")]
    #[value(name = "lightning_10")]
    Lightning10,
    #[serde(rename = "presentation_20")]
    #[value(name = "presentation_20")]
    Presentation20,
    #[serde(rename = "presentation_25")]
    #[value(name = "presentation_25")]
    Presentation25,
    #[serde(rename = "presentation_40")]
    #[value(name = "presentation_40")]
    Presentation40,
    #[serde(rename = "presentation_45")]
    #[value(name = "presentation_45")]
    Presentation45,
    #[serde(rename = "workshop_120")]
    #[value(name = "workshop_120")]
    Workshop120,
    #[serde(rename = "workshop_240")]
    #[value(name = "workshop_240")]
    Workshop240,
    /// Catch-all for unknown values from the API.
    #[value(skip)]
    #[serde(other)]
    Unknown,
}

impl<'de> Deserialize<'de> for ProposalFormat {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        enum Helper {
            #[serde(rename = "lightning_10")]
            Lightning10,
            #[serde(rename = "presentation_20")]
            Presentation20,
            #[serde(rename = "presentation_25")]
            Presentation25,
            #[serde(rename = "presentation_40")]
            Presentation40,
            #[serde(rename = "presentation_45")]
            Presentation45,
            #[serde(rename = "workshop_120")]
            Workshop120,
            #[serde(rename = "workshop_240")]
            Workshop240,
            #[serde(other)]
            Unknown,
        }
        Ok(match Helper::deserialize(deserializer)? {
            Helper::Lightning10 => Self::Lightning10,
            Helper::Presentation20 => Self::Presentation20,
            Helper::Presentation25 => Self::Presentation25,
            Helper::Presentation40 => Self::Presentation40,
            Helper::Presentation45 => Self::Presentation45,
            Helper::Workshop120 => Self::Workshop120,
            Helper::Workshop240 => Self::Workshop240,
            Helper::Unknown => Self::Unknown,
        })
    }
}

impl ProposalFormat {
    pub fn label(self) -> &'static str {
        match self {
            Self::Lightning10 => "Lightning 10min",
            Self::Presentation20 => "Talk 20min",
            Self::Presentation25 => "Talk 25min",
            Self::Presentation40 => "Talk 40min",
            Self::Presentation45 => "Talk 45min",
            Self::Workshop120 => "Workshop 2h",
            Self::Workshop240 => "Workshop 4h",
            Self::Unknown => "Unknown format",
        }
    }

    pub fn api_name(self) -> &'static str {
        match self {
            Self::Lightning10 => "lightning_10",
            Self::Presentation20 => "presentation_20",
            Self::Presentation25 => "presentation_25",
            Self::Presentation40 => "presentation_40",
            Self::Presentation45 => "presentation_45",
            Self::Workshop120 => "workshop_120",
            Self::Workshop240 => "workshop_240",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for ProposalFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.label())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    #[serde(rename = "_id")]
    pub id: String,
    pub title: String,
    pub status: ProposalStatus,
    #[serde(default)]
    pub format: Option<ProposalFormat>,
    #[serde(default)]
    pub level: serde_json::Value,
    #[serde(default)]
    pub language: serde_json::Value,
    #[serde(default, deserialize_with = "null_to_vec")]
    pub speakers: Vec<Speaker>,
    #[serde(default, deserialize_with = "null_to_vec")]
    pub topics: Vec<Topic>,
    #[serde(default, deserialize_with = "null_to_vec")]
    pub reviews: Vec<Review>,
    #[serde(default, rename = "_createdAt")]
    pub created_at: Option<String>,
    #[serde(default, rename = "_updatedAt")]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub outline: serde_json::Value,
    #[serde(default, deserialize_with = "null_to_vec")]
    pub description: Vec<serde_json::Value>,
    #[serde(default)]
    pub video: serde_json::Value,
}

/// Convert Portable Text blocks to plain text for terminal display.
pub fn portable_text_to_plain(blocks: &[serde_json::Value]) -> String {
    let mut paragraphs = Vec::new();

    for block in blocks {
        let block_type = block.get("_type").and_then(|t| t.as_str()).unwrap_or("");

        if block_type == "block" {
            let mut text = String::new();

            if let Some(children) = block.get("children").and_then(|c| c.as_array()) {
                for child in children {
                    if let Some(span_text) = child.get("text").and_then(|t| t.as_str()) {
                        text.push_str(span_text);
                    }
                }
            }

            // Treat list items with a bullet prefix
            let style = block.get("listItem").and_then(|l| l.as_str());
            if let Some("bullet") = style {
                text = format!("  • {text}");
            } else if let Some("number") = style {
                text = format!("  - {text}");
            }

            if !text.is_empty() {
                paragraphs.push(text);
            }
        }
    }

    paragraphs.join("\n\n")
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Topic {
    #[serde(flatten)]
    pub data: serde_json::Map<String, serde_json::Value>,
}

impl Topic {
    pub fn title(&self) -> Option<&str> {
        self.data.get("title").and_then(|v| v.as_str())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    #[serde(default)]
    pub score: Option<ReviewScore>,
    #[serde(default)]
    pub comment: serde_json::Value,
    #[serde(default)]
    pub reviewer: Option<Reviewer>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewScore {
    #[serde(default)]
    pub content: f64,
    #[serde(default)]
    pub relevance: f64,
    #[serde(default)]
    pub speaker: f64,
}

impl ReviewScore {
    pub fn total(&self) -> f64 {
        self.content + self.relevance + self.speaker
    }
}

/// Input for `proposal.admin.submitReview` mutation.
#[derive(Debug, Serialize)]
pub struct ReviewInput {
    pub id: String,
    pub comment: String,
    pub score: ReviewScore,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reviewer {
    pub name: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_full() {
        let json = serde_json::json!({
            "_id": "talk-1",
            "title": "Kubernetes Best Practices",
            "status": "submitted",
            "format": "presentation_25",
            "level": "intermediate",
            "language": "en",
            "outline": "We will cover...",
            "speakers": [
                {"_id": "sp-1", "name": "Alice", "email": "alice@example.com", "image": "https://img/a.jpg"}
            ],
            "topics": [
                {"title": "Kubernetes"},
                {"title": "DevOps"}
            ],
            "reviews": [
                {"score": {"content": 8.0, "relevance": 7.0, "speaker": 9.0}, "comment": "Great talk", "reviewer": {"name": "Bob"}}
            ]
        });

        let p: Proposal = serde_json::from_value(json).unwrap();
        assert_eq!(p.id, "talk-1");
        assert_eq!(p.title, "Kubernetes Best Practices");
        assert_eq!(p.status, ProposalStatus::Submitted);
        assert_eq!(p.format, Some(ProposalFormat::Presentation25));
        assert_eq!(p.speakers.len(), 1);
        assert_eq!(p.speakers[0].name, "Alice");
        assert_eq!(p.topics.len(), 2);
        assert_eq!(p.reviews.len(), 1);
        let score = p.reviews[0].score.as_ref().unwrap();
        assert!((score.total() - 24.0).abs() < f64::EPSILON);
        assert!((score.content - 8.0).abs() < f64::EPSILON);
    }

    #[test]
    fn deserialize_minimal() {
        let json = serde_json::json!({
            "_id": "talk-2",
            "title": "My Talk",
            "status": "draft"
        });

        let p: Proposal = serde_json::from_value(json).unwrap();
        assert_eq!(p.id, "talk-2");
        assert!(p.format.is_none());
        assert!(p.speakers.is_empty());
        assert!(p.topics.is_empty());
        assert!(p.reviews.is_empty());
        assert!(p.outline.is_null());
    }

    #[test]
    fn ignores_unknown_fields() {
        let json = serde_json::json!({
            "_id": "talk-3",
            "title": "Extra Fields Talk",
            "status": "accepted",
            "_createdAt": "2025-01-01T00:00:00Z",
            "_updatedAt": "2025-02-01T00:00:00Z",
            "description": [{"_type": "block", "children": []}],
            "scheduleInfo": {"room": "A", "slot": 1},
            "someNewField": "should be ignored"
        });

        let p: Proposal = serde_json::from_value(json).unwrap();
        assert_eq!(p.id, "talk-3");
        assert_eq!(p.status, ProposalStatus::Accepted);
    }

    #[test]
    fn review_without_reviewer() {
        let json = serde_json::json!({
            "score": {"content": 5.0, "relevance": 4.0, "speaker": 3.0},
            "comment": "Needs more detail"
        });

        let r: Review = serde_json::from_value(json).unwrap();
        let score = r.score.as_ref().unwrap();
        assert!((score.total() - 12.0).abs() < f64::EPSILON);
        assert!(r.reviewer.is_none());
    }

    #[test]
    fn review_without_score() {
        let json = serde_json::json!({
            "comment": "No score given",
            "reviewer": {"name": "Carol"}
        });

        let r: Review = serde_json::from_value(json).unwrap();
        assert!(r.score.is_none());
        assert_eq!(r.reviewer.as_ref().unwrap().name, "Carol");
    }

    #[test]
    fn portable_text_paragraphs() {
        let blocks = vec![
            serde_json::json!({
                "_type": "block",
                "children": [{"_type": "span", "text": "First paragraph."}]
            }),
            serde_json::json!({
                "_type": "block",
                "children": [
                    {"_type": "span", "text": "Second "},
                    {"_type": "span", "text": "paragraph."}
                ]
            }),
        ];
        let result = portable_text_to_plain(&blocks);
        assert_eq!(result, "First paragraph.\n\nSecond paragraph.");
    }

    #[test]
    fn portable_text_with_list_items() {
        let blocks = vec![
            serde_json::json!({
                "_type": "block",
                "children": [{"_type": "span", "text": "Intro"}]
            }),
            serde_json::json!({
                "_type": "block",
                "listItem": "bullet",
                "children": [{"_type": "span", "text": "Item one"}]
            }),
            serde_json::json!({
                "_type": "block",
                "listItem": "bullet",
                "children": [{"_type": "span", "text": "Item two"}]
            }),
        ];
        let result = portable_text_to_plain(&blocks);
        assert_eq!(result, "Intro\n\n  • Item one\n\n  • Item two");
    }

    #[test]
    fn portable_text_empty() {
        assert_eq!(portable_text_to_plain(&[]), "");
    }

    #[test]
    fn null_vec_fields_deserialize() {
        let json = serde_json::json!({
            "_id": "talk-null",
            "title": "Null Vecs",
            "status": "draft",
            "speakers": null,
            "topics": null,
            "reviews": null,
            "description": null
        });

        let p: Proposal = serde_json::from_value(json).unwrap();
        assert_eq!(p.id, "talk-null");
        assert!(p.speakers.is_empty());
        assert!(p.topics.is_empty());
        assert!(p.reviews.is_empty());
        assert!(p.description.is_empty());
    }

    #[test]
    fn unknown_status_deserializes_gracefully() {
        let json = serde_json::json!({
            "_id": "talk-future",
            "title": "Future Talk",
            "status": "archived",
            "speakers": [],
            "topics": [],
            "reviews": [],
        });

        let p: Proposal = serde_json::from_value(json).unwrap();
        assert_eq!(p.status, ProposalStatus::Unknown);
        assert_eq!(p.status.to_string(), "Unknown");
    }

    #[test]
    fn unknown_format_deserializes_gracefully() {
        let json = serde_json::json!({
            "_id": "talk-fmt",
            "title": "New Format",
            "status": "submitted",
            "format": "keynote_60",
            "speakers": [],
            "topics": [],
            "reviews": [],
        });

        let p: Proposal = serde_json::from_value(json).unwrap();
        assert_eq!(p.format, Some(ProposalFormat::Unknown));
    }
}
