use sendsure_rust::{
    demo_scenarios, evaluate, parse_http_request, ActionType, Decision, Intent, Registries,
};
use std::io::Read;

fn registry() -> Registries {
    Registries::default()
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
        transaction_origin: Some("demo".to_string()),
        asset_was_unsolicited: false,
    }
}

#[test]
fn corrected_xrp_tag_mismatch_stops_with_required_rule() {
    let scenario = &demo_scenarios()[0];
    let result = evaluate(&scenario.intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(
        result.triggered_rule_id,
        "TRANSFER_DESTINATION_TAG_MISMATCH"
    );
    assert_eq!(
        result.explanation,
        "The destination tag does not match the expected deposit tag for this account. Do not send until the deposit details are verified."
    );
}

#[test]
fn missing_xrp_tag_has_separate_stop_rule() {
    let mut intent = demo_scenarios()[6].intent.clone();
    intent.entered_destination_tag_or_memo = None;
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(result.triggered_rule_id, "TRANSFER_MISSING_DESTINATION_TAG");
}

#[test]
fn incorrect_xrp_tag_has_stop_rule() {
    let mut intent = demo_scenarios()[6].intent.clone();
    intent.entered_destination_tag_or_memo = Some("482109".to_string());
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(
        result.triggered_rule_id,
        "TRANSFER_DESTINATION_TAG_MISMATCH"
    );
}

#[test]
fn registry_derived_expected_tag_is_used_for_xrp_deposit() {
    let mut intent = demo_scenarios()[6].intent.clone();
    intent.expected_destination_tag_or_memo = None;
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Ready);
}

#[test]
fn demo_scenarios_follow_expected_decisions() {
    let scenarios = demo_scenarios();
    let expected_decisions = [
        Decision::Stop,
        Decision::Stop,
        Decision::Stop,
        Decision::Stop,
        Decision::Stop,
        Decision::Review,
        Decision::Ready,
    ];
    assert_eq!(scenarios.len(), expected_decisions.len());
    for (scenario, expected_decision) in scenarios.iter().zip(expected_decisions.iter()) {
        let result = evaluate(&scenario.intent, &registry());
        assert_eq!(
            result.decision, *expected_decision,
            "scenario {}",
            scenario.name
        );
    }
}

#[test]
fn seven_demo_scenarios_have_required_summary() {
    let scenarios = demo_scenarios();
    let decisions: Vec<_> = scenarios
        .iter()
        .map(|scenario| evaluate(&scenario.intent, &registry()).decision)
        .collect();
    assert_eq!(
        decisions.iter().filter(|d| **d == Decision::Stop).count(),
        5
    );
    assert_eq!(
        decisions.iter().filter(|d| **d == Decision::Review).count(),
        1
    );
    assert_eq!(
        decisions.iter().filter(|d| **d == Decision::Ready).count(),
        1
    );
}

#[test]
fn slippage_at_three_percent_is_ready() {
    let mut intent = demo_scenarios()[5].intent.clone();
    intent.swap_slippage_percent = Some(3.0);
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Ready);
}

#[test]
fn slippage_just_above_three_percent_is_review() {
    let mut intent = demo_scenarios()[5].intent.clone();
    intent.swap_slippage_percent = Some(3.01);
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Review);
}

#[test]
fn slippage_at_ten_percent_is_review() {
    let mut intent = demo_scenarios()[5].intent.clone();
    intent.swap_slippage_percent = Some(10.0);
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Review);
}

#[test]
fn slippage_above_ten_percent_is_stop() {
    let mut intent = demo_scenarios()[5].intent.clone();
    intent.swap_slippage_percent = Some(10.01);
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
}

#[test]
fn negative_slippage_is_stopped_with_invalid_slippage_rule() {
    let mut intent = demo_scenarios()[5].intent.clone();
    intent.swap_slippage_percent = Some(-1.0);
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(result.triggered_rule_id, "SWAP_INVALID_SLIPPAGE");
}

#[test]
fn zero_slippage_is_ready() {
    let mut intent = demo_scenarios()[5].intent.clone();
    intent.swap_slippage_percent = Some(0.0);
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Ready);
}

#[test]
fn destination_tag_with_trailing_whitespace_matches_expected_tag() {
    let mut intent = demo_scenarios()[6].intent.clone();
    intent.entered_destination_tag_or_memo = Some("482901 ".to_string());
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Ready);
}

#[test]
fn destination_tag_with_leading_whitespace_matches_expected_tag() {
    let mut intent = demo_scenarios()[6].intent.clone();
    intent.entered_destination_tag_or_memo = Some(" 482901".to_string());
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Ready);
}

#[test]
fn whitespace_only_destination_tag_is_treated_as_missing() {
    let mut intent = demo_scenarios()[6].intent.clone();
    intent.entered_destination_tag_or_memo = Some("   ".to_string());
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(result.triggered_rule_id, "TRANSFER_MISSING_DESTINATION_TAG");
}

#[test]
fn network_aliases_and_display_names_are_resolved_for_supported_tokens() {
    let mut registries = registry();
    registries.tokens.insert(
        "demo:bsc",
        sendsure_rust::Token {
            symbol: "DEMO",
            identifier: "demo:bsc",
            networks: std::collections::HashSet::from(["bnb-smart-chain"]),
        },
    );
    for (network_input, asset_identifier) in [
        ("xrpl", "xrp:xrp"),
        ("XRP Ledger", "xrp:xrp"),
        ("ethereum", "eth:usdc"),
        ("Ethereum", "eth:usdc"),
        ("base", "eth:usdc"),
        ("Base", "eth:usdc"),
        ("stellar", "xlm:xlm"),
        ("Stellar", "xlm:xlm"),
        ("  XrPl  ", "xrp:xrp"),
        ("  Ethereum  ", "eth:usdc"),
        ("bsc", "demo:bsc"),
        ("BNB Smart Chain", "demo:bsc"),
        ("  BSC  ", "demo:bsc"),
    ] {
        let intent = Intent {
            action_type: ActionType::Send,
            asset_identifier: Some(asset_identifier.to_string()),
            destination_network: Some(network_input.to_string()),
            destination_address: Some("0xDemoRecipient".to_string()),
            expected_destination_address: Some("0xDemoRecipient".to_string()),
            entered_destination_tag_or_memo: Some("482901".to_string()),
            expected_destination_tag_or_memo: Some("482901".to_string()),
            ..demo_scenarios()[6].intent.clone()
        };
        let result = evaluate(&intent, &registries);
        assert_eq!(
            result.decision,
            Decision::Ready,
            "network input {network_input}"
        );
    }
}

#[test]
fn xrp_ledger_alias_does_not_trigger_unsupported_destination_network() {
    let intent = Intent {
        action_type: ActionType::Send,
        asset_identifier: Some("xrp:xrp".to_string()),
        destination_network: Some("XRP Ledger".to_string()),
        destination_address: Some("rDemoExchangeAddress".to_string()),
        expected_destination_address: Some("rDemoExchangeAddress".to_string()),
        entered_destination_tag_or_memo: Some("482901".to_string()),
        expected_destination_tag_or_memo: Some("482901".to_string()),
        ..demo_scenarios()[6].intent.clone()
    };
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Ready);
}

#[test]
fn unknown_destination_network_does_not_fallback_to_source_network_for_usdc() {
    let intent = Intent {
        action_type: ActionType::Send,
        source_network: Some("ethereum".to_string()),
        destination_network: Some("tron".to_string()),
        asset_identifier: Some("eth:usdc".to_string()),
        destination_address: Some("0xDemoRecipient".to_string()),
        expected_destination_address: Some("0xDemoRecipient".to_string()),
        ..basic(ActionType::Send)
    };
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(
        result.triggered_rule_id,
        "TOKEN_UNKNOWN_DESTINATION_NETWORK"
    );
}

#[test]
fn misspelled_destination_network_stops_eth_transfer() {
    let intent = Intent {
        action_type: ActionType::Send,
        source_network: Some("ethereum".to_string()),
        destination_network: Some("etherum".to_string()),
        asset_identifier: Some("eth:eth".to_string()),
        destination_address: Some("0xDemoRecipient".to_string()),
        expected_destination_address: Some("0xDemoRecipient".to_string()),
        ..basic(ActionType::Send)
    };
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(
        result.triggered_rule_id,
        "TOKEN_UNKNOWN_DESTINATION_NETWORK"
    );
}

#[test]
fn blank_destination_network_can_fallback_to_source_network() {
    let intent = Intent {
        action_type: ActionType::Send,
        source_network: Some("ethereum".to_string()),
        destination_network: Some("   ".to_string()),
        asset_identifier: Some("eth:usdc".to_string()),
        destination_address: Some("0xDemoRecipient".to_string()),
        expected_destination_address: Some("0xDemoRecipient".to_string()),
        ..basic(ActionType::Send)
    };
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Ready);
}

#[test]
fn decimal_uint256_max_is_detected_as_unlimited_approval() {
    let mut intent = demo_scenarios()[4].intent.clone();
    intent.approval_amount_or_scope = Some(
        "115792089237316195423570985008687907853269984665640564039457584007913129639935"
            .to_string(),
    );
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(result.triggered_rule_id, "APPROVAL_UNLIMITED_ALLOWANCE");
}

#[test]
fn hex_uint256_max_is_detected_as_unlimited_approval() {
    let mut intent = demo_scenarios()[4].intent.clone();
    intent.approval_amount_or_scope =
        Some("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string());
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(result.triggered_rule_id, "APPROVAL_UNLIMITED_ALLOWANCE");
}

#[test]
fn hex_uint256_max_with_uppercase_prefix_is_detected_as_unlimited_approval() {
    let mut intent = demo_scenarios()[4].intent.clone();
    intent.approval_amount_or_scope =
        Some("0Xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string());
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(result.triggered_rule_id, "APPROVAL_UNLIMITED_ALLOWANCE");
}

#[test]
fn hex_uint256_max_with_uppercase_digits_after_lowercase_prefix_is_detected() {
    let mut intent = demo_scenarios()[4].intent.clone();
    intent.approval_amount_or_scope =
        Some("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF".to_string());
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(result.triggered_rule_id, "APPROVAL_UNLIMITED_ALLOWANCE");
}

#[test]
fn hex_uint256_max_with_uppercase_digits_after_uppercase_prefix_is_detected() {
    let mut intent = demo_scenarios()[4].intent.clone();
    intent.approval_amount_or_scope =
        Some("0XFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF".to_string());
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(result.triggered_rule_id, "APPROVAL_UNLIMITED_ALLOWANCE");
}

#[test]
fn stop_precedence_beats_review() {
    let mut intent = demo_scenarios()[5].intent.clone();
    intent.action_type = ActionType::Approve;
    intent.approval_amount_or_scope = Some("unlimited".to_string());
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(result.triggered_rule_id, "APPROVAL_UNLIMITED_ALLOWANCE");
}

#[test]
fn review_precedence_beats_ready() {
    let mut intent = demo_scenarios()[5].intent.clone();
    intent.swap_slippage_percent = Some(7.0);
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Review);
}

#[test]
fn empty_destination_address_is_stopped() {
    let mut intent = demo_scenarios()[6].intent.clone();
    intent.destination_address = None;
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(
        result.triggered_rule_id,
        "TRANSFER_EMPTY_DESTINATION_ADDRESS"
    );
}

#[test]
fn seed_phrase_request_is_stopped() {
    let mut intent = demo_scenarios()[6].intent.clone();
    intent.asset_symbol = Some("seed phrase".to_string());
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(
        result.triggered_rule_id,
        "SECURITY_SEED_OR_PRIVATE_KEY_REQUEST"
    );
}

#[test]
fn private_key_request_is_stopped() {
    let mut intent = demo_scenarios()[6].intent.clone();
    intent.asset_identifier = Some("private key".to_string());
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(
        result.triggered_rule_id,
        "SECURITY_SEED_OR_PRIVATE_KEY_REQUEST"
    );
}

#[test]
fn trusted_contract_is_allowed() {
    let intent = Intent {
        action_type: ActionType::Sign,
        asset_was_unsolicited: true,
        contract_address: Some("0xDemoSwapRouter".to_string()),
        transaction_origin: Some("airdrop-site.example".to_string()),
        ..demo_scenarios()[4].intent.clone()
    };
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Ready);
}

#[test]
fn unknown_contract_is_stopped() {
    let intent = Intent {
        action_type: ActionType::Sign,
        asset_was_unsolicited: true,
        contract_address: Some("0xUnexpectedAirdrop".to_string()),
        transaction_origin: Some("airdrop-site.example".to_string()),
        ..demo_scenarios()[4].intent.clone()
    };
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(
        result.triggered_rule_id,
        "SIGN_UNEXPECTED_AIRDROP_INTERACTION"
    );
}

#[test]
fn unsolicited_sign_without_contract_address_is_stopped() {
    let intent = Intent {
        action_type: ActionType::Sign,
        asset_was_unsolicited: true,
        contract_address: None,
        transaction_origin: Some("airdrop-site.example".to_string()),
        ..demo_scenarios()[4].intent.clone()
    };
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(
        result.triggered_rule_id,
        "SIGN_UNEXPECTED_AIRDROP_INTERACTION"
    );
}

#[test]
fn unsolicited_sign_with_blank_contract_address_is_stopped() {
    let intent = Intent {
        action_type: ActionType::Sign,
        asset_was_unsolicited: true,
        contract_address: Some("   ".to_string()),
        transaction_origin: Some("airdrop-site.example".to_string()),
        ..demo_scenarios()[4].intent.clone()
    };
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(
        result.triggered_rule_id,
        "SIGN_UNEXPECTED_AIRDROP_INTERACTION"
    );
}

#[test]
fn familiar_symbols_without_asset_identifier_stop() {
    for symbol in ["USDC", "USDT", "ETH", "BTC", "XRP", "SOL", "XLM"] {
        let intent = Intent {
            action_type: ActionType::Send,
            asset_symbol: Some(symbol.to_string()),
            asset_identifier: None,
            destination_address: Some("rDemoExchangeAddress".to_string()),
            expected_destination_address: Some("rDemoExchangeAddress".to_string()),
            entered_destination_tag_or_memo: Some("482901".to_string()),
            expected_destination_tag_or_memo: None,
            transaction_origin: Some("demo".to_string()),
            ..demo_scenarios()[6].intent.clone()
        };
        let result = evaluate(&intent, &registry());
        assert_eq!(result.decision, Decision::Stop, "symbol {symbol}");
        assert_eq!(
            result.triggered_rule_id, "TOKEN_MISSING_ASSET_IDENTIFIER",
            "symbol {symbol}"
        );
    }
}

#[test]
fn familiar_symbols_with_whitespace_and_missing_identifier_stop() {
    for symbol in ["USDC ", " USDC", " usdc "] {
        let intent = Intent {
            action_type: ActionType::Send,
            asset_symbol: Some(symbol.to_string()),
            asset_identifier: None,
            destination_address: Some("rDemoExchangeAddress".to_string()),
            expected_destination_address: Some("rDemoExchangeAddress".to_string()),
            entered_destination_tag_or_memo: Some("482901".to_string()),
            expected_destination_tag_or_memo: None,
            transaction_origin: Some("demo".to_string()),
            ..demo_scenarios()[6].intent.clone()
        };
        let result = evaluate(&intent, &registry());
        assert_eq!(result.decision, Decision::Stop, "symbol {symbol}");
        assert_eq!(
            result.triggered_rule_id, "TOKEN_MISSING_ASSET_IDENTIFIER",
            "symbol {symbol}"
        );
    }
}

#[test]
fn trusted_usdc_identifier_with_padded_symbol_whitespace_is_ready() {
    let intent = Intent {
        action_type: ActionType::Send,
        source_network: Some("ethereum".to_string()),
        destination_network: Some("base".to_string()),
        asset_symbol: Some(" USDC ".to_string()),
        asset_identifier: Some("eth:usdc".to_string()),
        destination_address: Some("0xDemoRecipient".to_string()),
        expected_destination_address: Some("0xDemoRecipient".to_string()),
        ..basic(ActionType::Send)
    };
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Ready);
}

#[test]
fn shortened_all_f_hexadecimal_is_not_treated_as_uint256_max() {
    let intent = Intent {
        action_type: ActionType::Approve,
        approval_amount_or_scope: Some("0xffff".to_string()),
        ..basic(ActionType::Approve)
    };
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Ready);
}

#[test]
fn parsing_http_request_handles_split_body() {
    struct ChunkedReader {
        data: Vec<u8>,
        chunk_size: usize,
        position: usize,
    }

    impl ChunkedReader {
        fn new(data: Vec<u8>, chunk_size: usize) -> Self {
            Self {
                data,
                chunk_size,
                position: 0,
            }
        }
    }

    impl Read for ChunkedReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.position >= self.data.len() {
                return Ok(0);
            }
            let size = self
                .chunk_size
                .min(self.data.len() - self.position)
                .min(buf.len());
            buf[..size].copy_from_slice(&self.data[self.position..self.position + size]);
            self.position += size;
            Ok(size)
        }
    }

    let request = b"POST /api/evaluate HTTP/1.1\r\nHost: example\r\nContent-Length: 16\r\n\r\n{\"hello\":\"x\"}";
    let mut reader = ChunkedReader::new(request.to_vec(), 5);
    let (header, body) = parse_http_request(&mut reader).unwrap();
    assert!(header.contains("Content-Length: 16"));
    assert_eq!(body, "{\"hello\":\"x\"}");
}

#[test]
fn parsing_http_request_handles_lowercase_content_length_header() {
    let request = b"POST /api/evaluate HTTP/1.1\r\nHost: example\r\ncontent-length: 16\r\n\r\n{\"hello\":\"x\"}";
    let mut reader = std::io::Cursor::new(request.as_slice());
    let (_header, body) = parse_http_request(&mut reader).unwrap();
    assert_eq!(body, "{\"hello\":\"x\"}");
}

#[test]
fn parsing_http_request_handles_mixed_case_content_length_header() {
    let request = b"POST /api/evaluate HTTP/1.1\r\nHost: example\r\nCoNtEnT-LeNgTh: 16\r\n\r\n{\"hello\":\"x\"}";
    let mut reader = std::io::Cursor::new(request.as_slice());
    let (_header, body) = parse_http_request(&mut reader).unwrap();
    assert_eq!(body, "{\"hello\":\"x\"}");
}

#[test]
fn ready_for_basic_matching_transfer() {
    let intent = Intent {
        expected_destination_tag_or_memo: None,
        entered_destination_tag_or_memo: Some("482901".to_string()),
        ..demo_scenarios()[6].intent.clone()
    };
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Ready);
}
