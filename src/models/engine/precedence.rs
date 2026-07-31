use super::super::decision::Decision;
use super::super::evaluation::RuleHit;

impl Decision {
    pub(crate) fn priority(self) -> u8 {
        match self {
            Decision::Ready => 0,
            Decision::Review => 1,
            Decision::Stop => 2,
        }
    }
}

pub(crate) fn hit(id: &str, decision: Decision, explanation: &str, step: &str) -> RuleHit {
    RuleHit {
        rule_id: id.into(),
        decision,
        explanation: explanation.into(),
        recommended_next_step: step.into(),
    }
}

pub(crate) fn sort_rule_hits(hits: &mut [RuleHit]) {
    hits.sort_by_key(|rule_hit| std::cmp::Reverse(rule_hit.decision.priority()));
}
