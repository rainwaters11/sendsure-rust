use crate::models::{ActionType, Decision, Intent, Scenario};

pub fn demo_scenarios() -> Vec<Scenario> {
    vec![
        scenario(
            "XRP destination tag mismatch",
            Decision::Stop,
            xrp_intent(Some("482109"), None),
        ),
        scenario(
            "USDC unsupported destination network",
            Decision::Stop,
            Intent {
                action_type: ActionType::Send,
                source_network: Some("ethereum".into()),
                destination_network: Some("solana".into()),
                asset_symbol: Some("USDC".into()),
                asset_identifier: Some("eth:usdc".into()),
                destination_address: Some("DemoSolanaAddress".into()),
                expected_destination_address: None,
                entered_destination_tag_or_memo: None,
                expected_destination_tag_or_memo: None,
                contract_address: None,
                approval_amount_or_scope: None,
                swap_slippage_percent: None,
                transaction_origin: Some("manual-entry".into()),
                asset_was_unsolicited: false,
            },
        ),
        scenario(
            "Unknown token using a familiar symbol",
            Decision::Stop,
            Intent {
                destination_address: Some("0xLookalikeRecipient".into()),
                asset_symbol: Some("USDC".into()),
                asset_identifier: Some("unknown:usdc".into()),
                ..basic(ActionType::Send)
            },
        ),
        scenario(
            "Unexpected airdrop interaction",
            Decision::Stop,
            Intent {
                action_type: ActionType::Sign,
                contract_address: Some("0xUnexpectedAirdrop".into()),
                transaction_origin: Some("airdrop-site.example".into()),
                asset_was_unsolicited: true,
                ..basic(ActionType::Sign)
            },
        ),
        scenario(
            "Unlimited approval",
            Decision::Stop,
            Intent {
                action_type: ActionType::Approve,
                approval_amount_or_scope: Some("unlimited".into()),
                contract_address: Some("0xDemoSwapRouter".into()),
                ..basic(ActionType::Approve)
            },
        ),
        scenario(
            "High-slippage swap at 7%",
            Decision::Review,
            Intent {
                action_type: ActionType::Swap,
                source_network: Some("ethereum".into()),
                destination_network: Some("ethereum".into()),
                asset_symbol: Some("DEMO".into()),
                asset_identifier: Some("eth:demo".into()),
                destination_address: Some("0xDemoRecipient".into()),
                expected_destination_address: Some("0xDemoRecipient".into()),
                swap_slippage_percent: Some(7.0),
                contract_address: Some("0xDemoSwapRouter".into()),
                ..basic(ActionType::Swap)
            },
        ),
        scenario(
            "Valid XRP with the correct destination tag",
            Decision::Ready,
            xrp_intent(Some("482901"), None),
        ),
    ]
}

fn scenario(name: &str, expected_decision: Decision, intent: Intent) -> Scenario {
    Scenario {
        name: name.into(),
        intent,
        expected_decision,
    }
}

fn basic(action_type: ActionType) -> Intent {
    Intent {
        action_type,
        source_network: None,
        destination_network: None,
        asset_symbol: None,
        asset_identifier: None,
        destination_address: None,
        expected_destination_address: None,
        entered_destination_tag_or_memo: None,
        expected_destination_tag_or_memo: None,
        contract_address: None,
        approval_amount_or_scope: None,
        swap_slippage_percent: None,
        transaction_origin: Some("demo".into()),
        asset_was_unsolicited: false,
    }
}

fn xrp_intent(entered: Option<&str>, expected: Option<&str>) -> Intent {
    Intent {
        action_type: ActionType::Send,
        source_network: Some("xrpl".into()),
        destination_network: Some("xrpl".into()),
        asset_symbol: Some("XRP".into()),
        asset_identifier: Some("xrp:xrp".into()),
        destination_address: Some("rDemoExchangeAddress".into()),
        expected_destination_address: Some("rDemoExchangeAddress".into()),
        entered_destination_tag_or_memo: entered.map(String::from),
        expected_destination_tag_or_memo: expected.map(String::from),
        contract_address: None,
        approval_amount_or_scope: None,
        swap_slippage_percent: None,
        transaction_origin: Some("Demo Exchange".into()),
        asset_was_unsolicited: false,
    }
}
