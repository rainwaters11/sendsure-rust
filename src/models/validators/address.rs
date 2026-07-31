use crate::models::registries::{DepositProfile, Registries};
use crate::models::Intent;

use super::network::{canonical_network_id, network_matches};

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

pub(crate) fn destination_addresses_match(
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

pub(crate) fn destination_matches_profile(
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

pub(crate) fn resolved_expected_tag(intent: &Intent, registries: &Registries) -> Option<String> {
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
