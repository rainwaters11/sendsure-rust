use crate::models::hit;
use crate::models::validators::{canonical_network_id, norm};
use crate::models::{ActionType, Decision, Intent, RuleHit};
use crate::registries::Registries;

pub(crate) fn token_swap_rules(i: &Intent, r: &Registries, hits: &mut Vec<RuleHit>) {
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
