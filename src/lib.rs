use std::io::{self, Read};

pub mod models;
pub mod registries;

pub use models::{evaluate, ActionType, Decision, Evaluation, Intent, RuleHit};
pub use registries::{ContractProfile, DepositProfile, Exchange, Network, Registries, Token};

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
