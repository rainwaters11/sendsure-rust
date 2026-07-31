mod cli;
mod decision;
mod engine;
mod evaluation;
pub mod frontend;
mod http;
mod intent;
mod registries;
mod rules;
mod scenario;
mod scenarios;
mod server;
mod validators;

pub use cli::run_demo;
pub use decision::Decision;
pub use engine::evaluate;
pub use evaluation::{Evaluation, RuleHit};
pub use http::parse_http_request;
pub use intent::{ActionType, Intent};
pub use registries::{ContractProfile, DepositProfile, Exchange, Network, Registries, Token};
pub use scenario::Scenario;
pub use scenarios::demo_scenarios;
pub use server::serve;

#[doc(hidden)]
pub use server::test_support;

pub(crate) use engine::hit;
