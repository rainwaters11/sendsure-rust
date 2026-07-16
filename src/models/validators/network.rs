use crate::registries::Registries;

pub(crate) fn norm(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|v| v.trim().to_ascii_lowercase())
        .filter(|v| !v.is_empty())
}

pub(crate) fn normalize_text(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-', '_'], "")
}

pub(crate) fn canonical_network_id(
    candidate: &str,
    registries: &Registries,
) -> Option<&'static str> {
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

pub(crate) fn network_matches(
    candidate: &str,
    profile_network: &str,
    registries: &Registries,
) -> bool {
    let candidate_id = canonical_network_id(candidate, registries);
    let profile_id = canonical_network_id(profile_network, registries);
    candidate_id
        .is_some_and(|candidate_id| profile_id.is_some_and(|profile_id| candidate_id == profile_id))
}
