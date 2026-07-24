use serde::{Deserialize, Serialize};

use super::decision::Decision;
use super::intent::Intent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub intent: Intent,
    pub expected_decision: Decision,
}
