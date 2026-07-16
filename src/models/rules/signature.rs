use crate::models::hit;
use crate::models::{ActionType, Decision, Intent, RuleHit};
use crate::registries::Registries;

pub(crate) fn signature_rules(i: &Intent, r: &Registries, hits: &mut Vec<RuleHit>) {
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
