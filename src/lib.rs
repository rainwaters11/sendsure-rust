use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{self, Read};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Decision {
    Ready,
    Review,
    Stop,
}

impl std::fmt::Display for Decision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Decision::Ready => "READY",
            Decision::Review => "REVIEW",
            Decision::Stop => "STOP",
        })
    }
}

impl Decision {
    fn priority(self) -> u8 {
        match self {
            Decision::Ready => 0,
            Decision::Review => 1,
            Decision::Stop => 2,
        }
    }
}

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

#[derive(Debug, Clone)]
pub struct Network {
    pub id: &'static str,
    pub display_name: &'static str,
    pub aliases: &'static [&'static str],
}
#[derive(Debug, Clone)]
pub struct Token {
    pub symbol: &'static str,
    pub identifier: &'static str,
    pub networks: HashSet<&'static str>,
}
#[derive(Debug, Clone)]
pub struct Exchange {
    pub name: &'static str,
}
#[derive(Debug, Clone)]
pub struct DepositProfile {
    pub exchange: &'static str,
    pub network: &'static str,
    pub destination_address: &'static str,
    pub expected_tag_or_memo: Option<&'static str>,
}
#[derive(Debug, Clone)]
pub struct ContractProfile {
    pub address: &'static str,
    pub name: &'static str,
    pub trusted: bool,
}

#[derive(Debug, Clone)]
pub struct Registries {
    pub networks: HashMap<&'static str, Network>,
    pub tokens: HashMap<&'static str, Token>,
    pub exchanges: HashMap<&'static str, Exchange>,
    pub deposits: Vec<DepositProfile>,
    pub contracts: HashMap<&'static str, ContractProfile>,
    pub familiar_symbols: HashSet<&'static str>,
}

impl Default for Registries {
    fn default() -> Self {
        let mut networks = HashMap::new();
        for (id, display_name, aliases) in [
            ("xrpl", "XRP Ledger", &[] as &[&str]),
            ("ethereum", "Ethereum", &[] as &[&str]),
            ("base", "Base", &[] as &[&str]),
            ("bnb-smart-chain", "BNB Smart Chain", &["bsc"] as &[&str]),
            ("stellar", "Stellar", &[] as &[&str]),
            ("polygon", "Polygon", &[] as &[&str]),
            ("solana", "Solana", &[] as &[&str]),
        ] {
            networks.insert(
                id,
                Network {
                    id,
                    display_name,
                    aliases,
                },
            );
        }
        let mut tokens = HashMap::new();
        tokens.insert(
            "xrp:xrp",
            Token {
                symbol: "XRP",
                identifier: "xrp:xrp",
                networks: HashSet::from(["xrpl"]),
            },
        );
        tokens.insert(
            "xlm:xlm",
            Token {
                symbol: "XLM",
                identifier: "xlm:xlm",
                networks: HashSet::from(["stellar"]),
            },
        );
        tokens.insert(
            "eth:usdc",
            Token {
                symbol: "USDC",
                identifier: "eth:usdc",
                networks: HashSet::from(["ethereum", "base"]),
            },
        );
        tokens.insert(
            "eth:eth",
            Token {
                symbol: "ETH",
                identifier: "eth:eth",
                networks: HashSet::from(["ethereum", "base"]),
            },
        );
        tokens.insert(
            "eth:demo",
            Token {
                symbol: "DEMO",
                identifier: "eth:demo",
                networks: HashSet::from(["ethereum"]),
            },
        );
        let exchanges = HashMap::from([(
            "demo-exchange",
            Exchange {
                name: "Demo Exchange",
            },
        )]);
        let deposits = vec![DepositProfile {
            exchange: "Demo Exchange",
            network: "XRP Ledger",
            destination_address: "rDemoExchangeAddress",
            expected_tag_or_memo: Some("482901"),
        }];
        let contracts = HashMap::from([
            (
                "0xDemoSwapRouter",
                ContractProfile {
                    address: "0xDemoSwapRouter",
                    name: "Demo Swap Router",
                    trusted: true,
                },
            ),
            (
                "0xUnexpectedAirdrop",
                ContractProfile {
                    address: "0xUnexpectedAirdrop",
                    name: "Unexpected Airdrop",
                    trusted: false,
                },
            ),
        ]);
        let familiar_symbols = HashSet::from(["USDC", "USDT", "ETH", "BTC", "XRP", "SOL", "XLM"]);
        Self {
            networks,
            tokens,
            exchanges,
            deposits,
            contracts,
            familiar_symbols,
        }
    }
}

pub fn evaluate(intent: &Intent, registries: &Registries) -> Evaluation {
    let mut hits = Vec::new();
    security_rules(intent, &mut hits);
    transfer_rules(intent, registries, &mut hits);
    token_swap_rules(intent, registries, &mut hits);
    signature_rules(intent, registries, &mut hits);
    approval_rules(intent, &mut hits);
    if hits.is_empty() {
        hits.push(hit(
            "READY_INTENT_MATCH",
            Decision::Ready,
            "The transaction details match the stated intent under deterministic SendSure rules.",
            "Continue only in your wallet after reviewing the final wallet prompt.",
        ));
    }
    hits.sort_by_key(|h| std::cmp::Reverse(h.decision.priority()));
    let top = hits[0].clone();
    Evaluation {
        decision: top.decision,
        triggered_rule_id: top.rule_id,
        explanation: top.explanation,
        recommended_next_step: top.recommended_next_step,
        rule_hits: hits,
    }
}

fn hit(id: &str, decision: Decision, explanation: &str, step: &str) -> RuleHit {
    RuleHit {
        rule_id: id.into(),
        decision,
        explanation: explanation.into(),
        recommended_next_step: step.into(),
    }
}

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

fn contains_sensitive_request(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.contains("seed phrase")
        || lowered.contains("seed-phrase")
        || lowered.contains("mnemonic")
        || lowered.contains("recovery phrase")
        || lowered.contains("private key")
        || lowered.contains("private-key")
        || lowered.contains("privkey")
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

fn security_rules(intent: &Intent, hits: &mut Vec<RuleHit>) {
    let values = [
        intent.source_network.as_deref(),
        intent.destination_network.as_deref(),
        intent.asset_symbol.as_deref(),
        intent.asset_identifier.as_deref(),
        intent.destination_address.as_deref(),
        intent.expected_destination_address.as_deref(),
        intent.entered_destination_tag_or_memo.as_deref(),
        intent.expected_destination_tag_or_memo.as_deref(),
        intent.contract_address.as_deref(),
        intent.approval_amount_or_scope.as_deref(),
        intent.transaction_origin.as_deref(),
    ];
    if values
        .iter()
        .any(|value| value.is_some_and(contains_sensitive_request))
    {
        hits.push(hit(
            "SECURITY_SEED_OR_PRIVATE_KEY_REQUEST",
            Decision::Stop,
            "The request appears to seek a seed phrase or private key, which SendSure does not support.",
            "Do not continue and keep the seed phrase or private key private.",
        ));
    }
}

fn transfer_rules(i: &Intent, registries: &Registries, hits: &mut Vec<RuleHit>) {
    if matches!(i.action_type, ActionType::Send | ActionType::Swap) {
        let has_destination = i
            .destination_address
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty());
        if !has_destination {
            hits.push(hit(
                "TRANSFER_EMPTY_DESTINATION_ADDRESS",
                Decision::Stop,
                "A destination address is required for SEND or SWAP requests.",
                "Provide the destination address before continuing.",
            ));
        }
    }
    if let (Some(a), Some(e)) = (&i.destination_address, &i.expected_destination_address) {
        if !destination_addresses_match(i, a, e, registries) {
            hits.push(hit(
                "TRANSFER_DESTINATION_ADDRESS_MISMATCH",
                Decision::Stop,
                "The destination address does not match the expected recipient.",
                "Do not send until the recipient address is verified.",
            ));
        }
    }
    if i.action_type == ActionType::Send {
        let expected = resolved_expected_tag(i, registries)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        if let Some(expected_tag) = expected {
            match &i.entered_destination_tag_or_memo {
                None => hits.push(hit("TRANSFER_MISSING_DESTINATION_TAG", Decision::Stop, "A destination tag or memo is required for this deposit account but was not entered.", "Do not send until the required tag or memo is added.")),
                Some(entered) => {
                    let entered_tag = entered.trim();
                    if entered_tag.is_empty() {
                        hits.push(hit("TRANSFER_MISSING_DESTINATION_TAG", Decision::Stop, "A destination tag or memo is required for this deposit account but was not entered.", "Do not send until the required tag or memo is added."));
                    } else if entered_tag != expected_tag {
                        hits.push(hit("TRANSFER_DESTINATION_TAG_MISMATCH", Decision::Stop, "The destination tag does not match the expected deposit tag for this account. Do not send until the deposit details are verified.", "Verify the deposit profile in the exchange and correct the destination tag before sending."));
                    }
                }
            }
        }
    }
}

fn token_swap_rules(i: &Intent, r: &Registries, hits: &mut Vec<RuleHit>) {
    if let Some(id) = norm(&i.asset_identifier) {
        if let Some(token) = r.tokens.get(id.as_str()) {
            if let Some(sym) = &i.asset_symbol {
                let symbol = sym.trim().to_ascii_uppercase();
                if !symbol.is_empty() && symbol != token.symbol {
                    hits.push(hit(
                        "TOKEN_ASSET_SYMBOL_MISMATCH",
                        Decision::Stop,
                        "The stated asset symbol does not match the registered token identifier.",
                        "Do not continue until the asset symbol and identifier match.",
                    ));
                }
            }
            if matches!(i.action_type, ActionType::Send | ActionType::Swap) {
                let source_network = i
                    .source_network
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty());
                let destination_network = i
                    .destination_network
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty());

                let resolved_source_network = if let Some(source_network) = source_network {
                    match canonical_network_id(source_network, r) {
                        Some(network_id) => Some(network_id),
                        None => {
                            hits.push(hit(
                                "TOKEN_UNKNOWN_SOURCE_NETWORK",
                                Decision::Stop,
                                "The source network is not recognized in SendSure's network registry.",
                                "Correct the source network before continuing.",
                            ));
                            None
                        }
                    }
                } else if destination_network.is_none() {
                    hits.push(hit(
                        "TOKEN_UNKNOWN_SOURCE_NETWORK",
                        Decision::Stop,
                        "The source network is required and must be recognized when destination network is not provided.",
                        "Provide a supported source network before continuing.",
                    ));
                    None
                } else {
                    None
                };

                if let Some(source_network_id) = resolved_source_network {
                    if !token.networks.contains(source_network_id) {
                        hits.push(hit(
                            "TOKEN_UNSUPPORTED_SOURCE_NETWORK",
                            Decision::Stop,
                            "The selected asset is not supported on the source network in SendSure's registry.",
                            "Choose a supported source network for this asset before continuing.",
                        ));
                    }
                }

                let resolved_destination_network = if let Some(destination_network) =
                    destination_network
                {
                    match canonical_network_id(destination_network, r) {
                        Some(network_id) => Some(network_id),
                        None => {
                            hits.push(hit(
                                "TOKEN_UNKNOWN_DESTINATION_NETWORK",
                                Decision::Stop,
                                "The destination network is not recognized in SendSure's network registry.",
                                "Correct the destination network before continuing.",
                            ));
                            None
                        }
                    }
                } else {
                    None
                };

                if let Some(destination_network_id) = resolved_destination_network {
                    if !token.networks.contains(destination_network_id) {
                        hits.push(hit(
                            "TOKEN_UNSUPPORTED_DESTINATION_NETWORK",
                            Decision::Stop,
                            "The selected asset is not supported on the destination network in SendSure's registry.",
                            "Choose a supported network for this asset before continuing.",
                        ));
                    }
                }
            }
        } else if let Some(sym) = &i.asset_symbol {
            let symbol = sym.trim().to_ascii_uppercase();
            if r.familiar_symbols.contains(symbol.as_str()) {
                hits.push(hit("TOKEN_UNKNOWN_FAMILIAR_SYMBOL", Decision::Stop, "An unknown token is using a familiar symbol, which can indicate a lookalike asset.", "Do not continue unless you independently verify the token identifier."));
            }
        }
    } else if let Some(sym) = &i.asset_symbol {
        let symbol = sym.trim().to_ascii_uppercase();
        if r.familiar_symbols.contains(symbol.as_str()) {
            hits.push(hit(
                "TOKEN_MISSING_ASSET_IDENTIFIER",
                Decision::Stop,
                "A familiar token symbol was provided without an asset identifier, which is ambiguous and unsafe.",
                "Provide the asset identifier before continuing.",
            ));
        }
    }
    if i.action_type == ActionType::Swap {
        if let Some(slip) = i.swap_slippage_percent {
            if !slip.is_finite() || slip < 0.0 {
                hits.push(hit(
                    "SWAP_INVALID_SLIPPAGE",
                    Decision::Stop,
                    "The swap slippage is negative or invalid.",
                    "Enter a valid slippage percentage before proceeding.",
                ));
            } else if slip <= 3.0 {
                // READY threshold: 0% through 3%.
            } else if slip <= 10.0 {
                hits.push(hit(
                    "SWAP_SLIPPAGE_REVIEW",
                    Decision::Review,
                    "The swap slippage is above the READY threshold and needs review.",
                    "Review price impact and lower slippage if this was not intentional.",
                ));
            } else {
                hits.push(hit(
                    "SWAP_SLIPPAGE_STOP",
                    Decision::Stop,
                    "The swap slippage is above the acceptable threshold.",
                    "Do not proceed with this swap at the current slippage.",
                ));
            }
        } else {
            hits.push(hit(
                "SWAP_MISSING_SLIPPAGE",
                Decision::Review,
                "SendSure cannot complete the slippage-risk check until swap slippage tolerance is supplied.",
                "Provide slippage tolerance before proceeding with this swap.",
            ));
        }
    }
}

fn signature_rules(i: &Intent, r: &Registries, hits: &mut Vec<RuleHit>) {
    if i.action_type == ActionType::Sign && i.asset_was_unsolicited {
        let contract_address = i
            .contract_address
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let is_untrusted_or_missing = contract_address.is_none()
            || contract_address.is_some_and(|address| {
                r.contracts
                    .get(address)
                    .map(|profile| !profile.trusted)
                    .unwrap_or(true)
            });
        if is_untrusted_or_missing {
            hits.push(hit(
                "SIGN_UNEXPECTED_AIRDROP_INTERACTION",
                Decision::Stop,
                "The transaction interacts with an unsolicited asset or unexpected airdrop contract.",
                "Do not sign unsolicited airdrop transactions or messages.",
            ));
        }
    }
}

fn approval_rules(i: &Intent, hits: &mut Vec<RuleHit>) {
    if i.action_type == ActionType::Approve {
        let scope = i
            .approval_amount_or_scope
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if scope.is_none() {
            hits.push(hit(
                "APPROVAL_MISSING_SCOPE",
                Decision::Stop,
                "SendSure cannot determine whether the approval is limited or unlimited until an amount or scope is supplied.",
                "Provide an approval amount or scope before continuing.",
            ));
            return;
        }
        if let Some(scope) = scope {
            let s = scope.to_ascii_lowercase();
            if ["unlimited", "infinite", "max", "maximum"]
                .iter()
                .any(|w| s.contains(w))
                || s.contains("uint256::max")
                || s.contains("u256::max")
                || is_uint256_max(scope)
            {
                hits.push(hit(
                    "APPROVAL_UNLIMITED_ALLOWANCE",
                    Decision::Stop,
                    "The approval grants an unlimited token allowance.",
                    "Use a limited approval amount or reject the request.",
                ));
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
