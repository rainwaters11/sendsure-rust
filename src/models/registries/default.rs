use std::collections::{HashMap, HashSet};

use super::types::{ContractProfile, DepositProfile, Exchange, Network, Registries, Token};

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
