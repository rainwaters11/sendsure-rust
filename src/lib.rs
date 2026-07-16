use std::io::{self, Read};

pub mod models;
pub mod registries;

pub use models::{evaluate, ActionType, Decision, Evaluation, Intent, RuleHit};
pub use registries::{ContractProfile, DepositProfile, Exchange, Network, Registries, Token};

fn norm(s: &Option<String>) -> Option<String> {
    s.as_ref()
        .map(|v| v.trim().to_ascii_lowercase())
        .filter(|v| !v.is_empty())
}

fn normalize_text(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-', '_'], "")
}

fn canonical_network_id(candidate: &str, registries: &Registries) -> Option<&'static str> {
    let normalized_candidate = normalize_text(candidate);
    registries.networks.values().find_map(|network| {
        let network_id = normalize_text(network.id);
        let display_name = normalize_text(network.display_name);
        let alias_matches = network
            .aliases
            .iter()
            .any(|alias| normalize_text(alias) == normalized_candidate);
        (network_id == normalized_candidate
            || display_name == normalized_candidate
            || alias_matches)
            .then_some(network.id)
    })
}

fn network_matches(candidate: &str, profile_network: &str, registries: &Registries) -> bool {
    let candidate_id = canonical_network_id(candidate, registries);
    let profile_id = canonical_network_id(profile_network, registries);
    candidate_id
        .is_some_and(|candidate_id| profile_id.is_some_and(|profile_id| candidate_id == profile_id))
}

fn is_evm_network(network_id: &str) -> bool {
    matches!(network_id, "ethereum" | "base" | "bnb-smart-chain")
}

fn normalized_context_network_id(intent: &Intent, registries: &Registries) -> Option<&'static str> {
    intent
        .destination_network
        .as_deref()
        .and_then(|network| canonical_network_id(network, registries))
        .or_else(|| {
            intent
                .source_network
                .as_deref()
                .and_then(|network| canonical_network_id(network, registries))
        })
}

fn destination_addresses_match(
    intent: &Intent,
    entered: &str,
    expected: &str,
    registries: &Registries,
) -> bool {
    let entered = entered.trim();
    let expected = expected.trim();
    match normalized_context_network_id(intent, registries) {
        Some(network_id) if is_evm_network(network_id) => entered.eq_ignore_ascii_case(expected),
        _ => entered == expected,
    }
}

fn destination_matches_profile(
    destination_address: &str,
    deposit: &DepositProfile,
    registries: &Registries,
) -> bool {
    let is_xrpl = canonical_network_id(deposit.network, registries) == Some("xrpl");
    if is_xrpl {
        destination_address == deposit.destination_address
    } else {
        deposit
            .destination_address
            .eq_ignore_ascii_case(destination_address)
    }
}

fn resolved_expected_tag(intent: &Intent, registries: &Registries) -> Option<String> {
    let destination_address = intent
        .destination_address
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    let network_matches = intent
        .destination_network
        .as_deref()
        .is_some_and(|network| {
            registries.deposits.iter().any(|deposit| {
                destination_matches_profile(destination_address, deposit, registries)
                    && network_matches(network, deposit.network, registries)
            })
        })
        || intent.source_network.as_deref().is_some_and(|network| {
            registries.deposits.iter().any(|deposit| {
                destination_matches_profile(destination_address, deposit, registries)
                    && network_matches(network, deposit.network, registries)
            })
        });
    if network_matches {
        return registries.deposits.iter().find_map(|deposit| {
            if destination_matches_profile(destination_address, deposit, registries) {
                deposit
                    .expected_tag_or_memo
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
            } else {
                None
            }
        });
    }
    intent
        .expected_destination_tag_or_memo
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn is_uint256_max(value: &str) -> bool {
    let trimmed = value.trim();
    let bytes = if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        parse_hex_to_le_bytes(hex)
    } else {
        parse_decimal_to_le_bytes(trimmed)
    };
    bytes.map(|value| value == vec![0xff; 32]).unwrap_or(false)
}

fn parse_decimal_to_le_bytes(value: &str) -> Option<Vec<u8>> {
    let mut bytes = vec![0_u8];
    for ch in value.chars() {
        let digit = ch.to_digit(10)? as u16;
        let mut carry = digit;
        for byte in &mut bytes {
            let sum = (*byte as u16) * 10 + carry;
            *byte = (sum & 0xff) as u8;
            carry = sum >> 8;
        }
        while carry > 0 {
            bytes.push((carry & 0xff) as u8);
            carry >>= 8;
        }
    }
    Some(normalize_le_bytes(bytes))
}

fn parse_hex_to_le_bytes(value: &str) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut current = 0_u8;
    let mut nibble_index = 0_u8;
    for ch in value.chars() {
        let nibble = ch.to_digit(16)? as u8;
        if nibble_index.is_multiple_of(2) {
            current = nibble << 4;
        } else {
            current |= nibble;
            bytes.push(current);
        }
        nibble_index += 1;
    }
    if nibble_index % 2 == 1 {
        bytes.push(current);
    }
    Some(normalize_le_bytes(bytes))
}

fn normalize_le_bytes(mut bytes: Vec<u8>) -> Vec<u8> {
    while bytes.len() > 1 && bytes.last() == Some(&0) {
        bytes.pop();
    }
    bytes
}

pub fn parse_http_request<R: Read>(reader: &mut R) -> io::Result<(String, String)> {
    let mut buffer = Vec::new();
    let mut read_buf = [0_u8; 4096];
    loop {
        let bytes_read = reader.read(&mut read_buf)?;
        if bytes_read == 0 {
            break;
        }
        buffer.extend_from_slice(&read_buf[..bytes_read]);
        if let Some(header_end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            let header_bytes = buffer[..header_end].to_vec();
            let body_start = header_end + 4;
            let mut body = buffer[body_start..].to_vec();
            let header_text = String::from_utf8_lossy(&header_bytes).into_owned();
            if let Some(content_length) = parse_content_length(&header_text) {
                while body.len() < content_length {
                    let bytes_read = reader.read(&mut read_buf)?;
                    if bytes_read == 0 {
                        return Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "incomplete HTTP body for declared Content-Length",
                        ));
                    }
                    body.extend_from_slice(&read_buf[..bytes_read]);
                }
                body.truncate(content_length);
            }
            return Ok((header_text, String::from_utf8_lossy(&body).into_owned()));
        }
    }
    let header = String::from_utf8_lossy(&buffer).into_owned();
    Ok((header, String::new()))
}

fn parse_content_length(header_text: &str) -> Option<usize> {
    header_text.lines().find_map(|line| {
        let trimmed = line.trim();
        let (name, value) = trimmed.split_once(':')?;
        if name.trim().eq_ignore_ascii_case("content-length") {
            value.trim().parse::<usize>().ok()
        } else {
            None
        }
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Scenario {
    pub name: String,
    pub intent: Intent,
    pub expected_decision: Decision,
}

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
