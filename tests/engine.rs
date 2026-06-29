use sendsure_rust::{demo_scenarios, evaluate, ActionType, Decision, Intent, Registries};

fn registry() -> Registries {
    Registries::default()
}

#[test]
fn corrected_xrp_tag_mismatch_stops_with_required_rule() {
    let scenario = &demo_scenarios()[0];
    let result = evaluate(&scenario.intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(
        result.triggered_rule_id,
        "TRANSFER_DESTINATION_TAG_MISMATCH"
    );
    assert_eq!(result.explanation, "The destination tag does not match the expected deposit tag for this account. Do not send until the deposit details are verified.");
}

#[test]
fn missing_xrp_tag_has_separate_stop_rule() {
    let mut intent = demo_scenarios()[6].intent.clone();
    intent.entered_destination_tag_or_memo = None;
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(result.triggered_rule_id, "TRANSFER_MISSING_DESTINATION_TAG");
}

#[test]
fn seven_demo_scenarios_have_required_order_and_summary() {
    let scenarios = demo_scenarios();
    let decisions: Vec<_> = scenarios
        .iter()
        .map(|s| evaluate(&s.intent, &registry()).decision)
        .collect();
    assert_eq!(
        decisions,
        vec![
            Decision::Stop,
            Decision::Stop,
            Decision::Stop,
            Decision::Stop,
            Decision::Stop,
            Decision::Review,
            Decision::Ready
        ]
    );
    assert_eq!(
        decisions.iter().filter(|d| **d == Decision::Stop).count(),
        5
    );
    assert_eq!(
        decisions.iter().filter(|d| **d == Decision::Review).count(),
        1
    );
    assert_eq!(
        decisions.iter().filter(|d| **d == Decision::Ready).count(),
        1
    );
}

#[test]
fn stop_precedence_beats_review() {
    let mut intent = demo_scenarios()[5].intent.clone();
    intent.action_type = ActionType::Approve;
    intent.approval_amount_or_scope = Some("unlimited".to_string());
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Stop);
    assert_eq!(result.triggered_rule_id, "APPROVAL_UNLIMITED_ALLOWANCE");
}

#[test]
fn ready_for_basic_matching_transfer() {
    let intent = Intent {
        expected_destination_tag_or_memo: None,
        entered_destination_tag_or_memo: None,
        ..demo_scenarios()[6].intent.clone()
    };
    let result = evaluate(&intent, &registry());
    assert_eq!(result.decision, Decision::Ready);
}
