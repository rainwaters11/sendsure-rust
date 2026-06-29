use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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
        for (id, display_name) in [
            ("xrpl", "XRP Ledger"),
            ("ethereum", "Ethereum"),
            ("polygon", "Polygon"),
            ("solana", "Solana"),
        ] {
            networks.insert(id, Network { id, display_name });
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
            "eth:usdc",
            Token {
                symbol: "USDC",
                identifier: "eth:usdc",
                networks: HashSet::from(["ethereum", "polygon"]),
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
        let familiar_symbols = HashSet::from(["USDC", "USDT", "ETH", "BTC", "XRP", "SOL"]);
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

fn transfer_rules(i: &Intent, _r: &Registries, hits: &mut Vec<RuleHit>) {
    if let (Some(a), Some(e)) = (&i.destination_address, &i.expected_destination_address) {
        if a != e {
            hits.push(hit(
                "TRANSFER_DESTINATION_ADDRESS_MISMATCH",
                Decision::Stop,
                "The destination address does not match the expected recipient.",
                "Do not send until the recipient address is verified.",
            ));
        }
    }
    if i.action_type == ActionType::Send {
        if let Some(expected) = &i.expected_destination_tag_or_memo {
            match &i.entered_destination_tag_or_memo {
                None => hits.push(hit("TRANSFER_MISSING_DESTINATION_TAG", Decision::Stop, "A destination tag or memo is required for this deposit account but was not entered.", "Do not send until the required tag or memo is added.")),
                Some(entered) if entered != expected => hits.push(hit("TRANSFER_DESTINATION_TAG_MISMATCH", Decision::Stop, "The destination tag does not match the expected deposit tag for this account. Do not send until the deposit details are verified.", "Verify the deposit profile in the exchange and correct the destination tag before sending.")),
                _ => {}
            }
        }
    }
}
fn token_swap_rules(i: &Intent, r: &Registries, hits: &mut Vec<RuleHit>) {
    if let Some(id) = norm(&i.asset_identifier) {
        if let Some(token) = r.tokens.get(id.as_str()) {
            if let Some(net) = norm(&i.destination_network).or_else(|| norm(&i.source_network)) {
                if !token.networks.contains(net.as_str()) {
                    hits.push(hit("TOKEN_UNSUPPORTED_DESTINATION_NETWORK", Decision::Stop, "The selected asset is not supported on the destination network in SendSure's registry.", "Choose a supported network for this asset before continuing."));
                }
            }
        } else if let Some(sym) = &i.asset_symbol {
            if r.familiar_symbols
                .contains(sym.to_ascii_uppercase().as_str())
            {
                hits.push(hit("TOKEN_UNKNOWN_FAMILIAR_SYMBOL", Decision::Stop, "An unknown token is using a familiar symbol, which can indicate a lookalike asset.", "Do not continue unless you independently verify the token identifier."));
            }
        }
    }
    if i.action_type == ActionType::Swap {
        if let Some(slip) = i.swap_slippage_percent {
            if slip > 5.0 {
                hits.push(hit(
                    "SWAP_HIGH_SLIPPAGE",
                    Decision::Review,
                    "The swap slippage is high enough to materially change the received amount.",
                    "Review price impact and lower slippage if this was not intentional.",
                ));
            }
        }
    }
}
fn signature_rules(i: &Intent, r: &Registries, hits: &mut Vec<RuleHit>) {
    if i.action_type == ActionType::Sign && i.asset_was_unsolicited {
        if let Some(c) = &i.contract_address {
            if r.contracts
                .get(c.as_str())
                .map(|p| !p.trusted)
                .unwrap_or(true)
            {
                hits.push(hit("SIGN_UNEXPECTED_AIRDROP_INTERACTION", Decision::Stop, "The transaction interacts with an unsolicited asset or unexpected airdrop contract.", "Do not sign unsolicited airdrop transactions or messages."));
            }
        }
    }
}
fn approval_rules(i: &Intent, hits: &mut Vec<RuleHit>) {
    if i.action_type == ActionType::Approve {
        if let Some(scope) = &i.approval_amount_or_scope {
            let s = scope.to_ascii_lowercase();
            if ["unlimited", "infinite", "max", "maximum"]
                .iter()
                .any(|w| s.contains(w))
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
            xrp_intent(Some("482109"), Some("482901")),
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
                swap_slippage_percent: Some(7.0),
                contract_address: Some("0xDemoSwapRouter".into()),
                ..basic(ActionType::Swap)
            },
        ),
        scenario(
            "Valid XRP with the correct destination tag",
            Decision::Ready,
            xrp_intent(Some("482901"), Some("482901")),
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
