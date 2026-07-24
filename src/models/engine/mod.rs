mod precedence;

use super::registries::Registries;
use super::rules::{
    approval_rules, security_rules, signature_rules, token_swap_rules, transfer_rules,
};
use super::{Decision, Evaluation, Intent};

pub(crate) use precedence::{hit, sort_rule_hits};

pub fn evaluate(intent: &Intent, registries: &Registries) -> Evaluation {
    let mut hits = Vec::new();
    security_rules(intent, &mut hits);
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
    sort_rule_hits(&mut hits);
    let top = hits[0].clone();
    Evaluation {
        decision: top.decision,
        triggered_rule_id: top.rule_id,
        explanation: top.explanation,
        recommended_next_step: top.recommended_next_step,
        rule_hits: hits,
    }
}
