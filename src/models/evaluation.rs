use serde::{Deserialize, Serialize};

use super::decision::Decision;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleHit {
    pub rule_id: String,
    pub decision: Decision,
    pub explanation: String,
    pub recommended_next_step: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evaluation {
    pub decision: Decision,
    pub triggered_rule_id: String,
    pub explanation: String,
    pub recommended_next_step: String,
    pub rule_hits: Vec<RuleHit>,
}
