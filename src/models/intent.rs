use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActionType {
    Send,
    Swap,
    Approve,
    Sign,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intent {
    pub action_type: ActionType,
    pub source_network: Option<String>,
    pub destination_network: Option<String>,
    pub asset_symbol: Option<String>,
    pub asset_identifier: Option<String>,
    pub destination_address: Option<String>,
    pub expected_destination_address: Option<String>,
    pub entered_destination_tag_or_memo: Option<String>,
    pub expected_destination_tag_or_memo: Option<String>,
    pub contract_address: Option<String>,
    pub approval_amount_or_scope: Option<String>,
    pub swap_slippage_percent: Option<f64>,
    pub transaction_origin: Option<String>,
    pub asset_was_unsolicited: bool,
}
