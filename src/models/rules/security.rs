use crate::models::hit;
use crate::models::{Decision, Intent, RuleHit};

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

pub(crate) fn security_rules(intent: &Intent, hits: &mut Vec<RuleHit>) {
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
