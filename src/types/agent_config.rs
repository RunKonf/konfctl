use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    pub conference_context: Option<String>,
    pub proposal_review_config: Option<String>,
    pub sponsor_crm_config: Option<String>,
}
