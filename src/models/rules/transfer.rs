use crate::models::hit;
use crate::models::validators::{destination_addresses_match, resolved_expected_tag};
use crate::models::{ActionType, Decision, Intent, RuleHit};
use crate::models::registries::Registries;

pub(crate) fn transfer_rules(i: &Intent, registries: &Registries, hits: &mut Vec<RuleHit>) {
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
