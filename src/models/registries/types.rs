use std::collections::{HashMap, HashSet};

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
