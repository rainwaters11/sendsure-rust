use crate::is_uint256_max;
use crate::models::hit;
use crate::models::{ActionType, Decision, Intent, RuleHit};

pub(crate) fn approval_rules(i: &Intent, hits: &mut Vec<RuleHit>) {
    if i.action_type == ActionType::Approve {
        let scope = i
            .approval_amount_or_scope
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if scope.is_none() {
            hits.push(hit(
                "APPROVAL_MISSING_SCOPE",
                Decision::Stop,
                "SendSure cannot determine whether the approval is limited or unlimited until an amount or scope is supplied.",
                "Provide an approval amount or scope before continuing.",
            ));
            return;
        }
        if let Some(scope) = scope {
            let s = scope.to_ascii_lowercase();
            if ["unlimited", "infinite", "max", "maximum"]
                .iter()
                .any(|w| s.contains(w))
                || s.contains("uint256::max")
                || s.contains("u256::max")
                || is_uint256_max(scope)
            {
                hits.push(hit(
                    "APPROVAL_UNLIMITED_ALLOWANCE",
                    Decision::Stop,
                    "The approval grants an unlimited token allowance.",
                    "Use a limited approval amount or reject the request.",
                ));
            }
        }
    }
}
