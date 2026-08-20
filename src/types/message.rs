use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationCounterpart {
    pub name: String,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationLastMessage {
    pub author_id: String,
    pub author_name: Option<String>,
    pub excerpt: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationRow {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(default)]
    pub archived: bool,
    pub assigned_to: Option<String>,
    pub conversation_type: String,
    pub counterpart: Option<ConversationCounterpart>,
    pub created_at: String,
    pub direct: bool,
    pub last_message: Option<ConversationLastMessage>,
    pub last_message_at: String,
    #[serde(default)]
    pub muted: bool,
    #[serde(default)]
    pub needs_reply: bool,
    pub proposal_id: Option<String>,
    pub proposal_title: Option<String>,
    pub status: String,
    pub subject: Option<String>,
    #[serde(default)]
    pub unread_count: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationParticipant {
    #[serde(rename = "_id", default)]
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(default)]
    pub is_organizer: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationMessage {
    #[serde(rename = "_id")]
    pub id: String,
    pub author_id: String,
    #[serde(default)]
    pub author_name: Option<String>,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationPreference {
    #[serde(default)]
    pub muted: bool,
    pub email_override: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationDetail {
    #[serde(rename = "_id")]
    pub id: String,
    pub subject: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetConversationResult {
    pub conversation: ConversationDetail,
    #[serde(default)]
    pub messages: Vec<ConversationMessage>,
    #[serde(default)]
    pub participants: Vec<ConversationParticipant>,
    pub preference: Option<ConversationPreference>,
}
