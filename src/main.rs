use sendsure_rust::{demo_scenarios, evaluate, serve, Decision, Registries};

fn main() {
    if std::env::args().any(|a| a == "serve") {
        let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let address = format!("0.0.0.0:{port}");
        if let Err(error) = serve(&address) {
            eprintln!("server error: {error}");
            std::process::exit(1);
        }
    } else {
        run_demo();
    }
}

fn run_demo() {
    let registries = Registries::default();
    let scenarios = demo_scenarios();
    let mut stop = 0;
    let mut review = 0;
    let mut ready = 0;
    println!("SendSure deterministic Rust preflight demo");
    println!("Rules are deterministic Rust code; no LLM, blockchain API, database, or external risk service is used.\n");
    for (index, scenario) in scenarios.iter().enumerate() {
        let result = evaluate(&scenario.intent, &registries);
        match result.decision {
            Decision::Stop => stop += 1,
            Decision::Review => review += 1,
            Decision::Ready => ready += 1,
        }
        println!("{}. {} → {}", index + 1, scenario.name, result.decision);
        println!("   Rule: {}", result.triggered_rule_id);
        println!("   {}", result.explanation);
        println!("   Next: {}\n", result.recommended_next_step);
    }
    println!("Summary");
    println!("STOP: {stop}");
    println!("REVIEW: {review}");
    println!("READY: {ready}");
    println!("Total scenarios: {}", scenarios.len());
}

#[cfg(test)]
mod tests {
    use sendsure_rust::frontend::{APP_JS, INDEX_HTML, STYLES_CSS};

    fn css_without_whitespace(source: &str) -> String {
        source.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn frontend_declares_favicon_links_for_browser_tab() {
        assert!(
            !INDEX_HTML.contains("href=\"/favicon.ico\""),
            "do not advertise /favicon.ico without a real ICO/PNG asset"
        );
        assert!(
            INDEX_HTML.contains(
                "<link rel=\"icon\" href=\"/assets/sendsure-mark.svg\" type=\"image/svg+xml\">"
            ),
            "svg favicon should reference the approved shield mark"
        );
    }

    #[test]
    fn frontend_contains_required_form_and_fetch_guards() {
        assert!(
            INDEX_HTML.contains("<button type=\"button\" id=\"reset\">Reset</button>"),
            "reset button should be visible and non-submit"
        );
        assert!(
            INDEX_HTML
                .contains("<button type=\"button\" id=\"check\">Run preflight check</button>"),
            "check button should not submit the form"
        );
        assert!(
            APP_JS.contains("checkButton.addEventListener('click', async () => {")
                && APP_JS.contains("await evaluateFromForm();"),
            "top check button should trigger the same preflight evaluation flow"
        );
        assert!(
            INDEX_HTML
                .contains("<button type=\"button\" id=\"continue\" disabled>Continue</button>"),
            "continue should be non-submit and disabled initially"
        );
        assert!(
            INDEX_HTML.contains("<button type=\"button\" data-action=\"SEND\""),
            "action tabs should not submit the form"
        );
        assert!(
            APP_JS.contains("form.addEventListener('submit'"),
            "form should use one explicit submit handler"
        );
        assert!(
            !APP_JS.contains("form.onsubmit"),
            "legacy onsubmit handler should not exist"
        );
        assert!(
            APP_JS.contains("function buildIntentFromForm()"),
            "intent must be rebuilt from live form state"
        );
        assert!(
            APP_JS.contains("await evaluateIntent(buildIntentFromForm())"),
            "submit path must evaluate a fresh serialized intent"
        );
        assert!(
            APP_JS.contains("cache: 'no-store'"),
            "evaluate fetch should bypass cache"
        );
        assert!(
            APP_JS.contains("button.type = 'button';"),
            "scenario buttons should be explicitly non-submit"
        );
        assert!(
            APP_JS.contains("SWAP: [")
                && APP_JS.contains("'destination_address'")
                && APP_JS.contains("'expected_destination_address'"),
            "swap action must include destination fields required by engine input"
        );
        assert!(
            APP_JS.contains("formFields.action_type.addEventListener('change'"),
            "action dropdown should share the same action-state handler"
        );
        assert!(
            APP_JS.contains("applyManualActionChange(button.dataset.action || 'SEND')"),
            "action tabs should use shared action-state handler"
        );
        assert!(
            APP_JS.contains("const nextAction = normalizeAction(action);")
                && APP_JS.contains("if (nextAction === selectedAction) {")
                && APP_JS.contains("setActionState(nextAction, { clearIrrelevant: true, focus: true });")
                && !APP_JS.contains("setActionState(action, { clearAll: true, focus: true });"),
            "re-clicking the active action should not clear fields; only real action changes should clear irrelevant fields"
        );
        assert!(
            APP_JS.contains("const actionChangedText = 'Transaction details changed. Run the preflight check again.';"),
            "manual action changes should invalidate previous result copy"
        );
        assert!(
            APP_JS.contains("function invalidateActionEvaluationState()")
                && APP_JS.contains("clearPendingEvaluation();")
                && APP_JS.contains("selectedScenarioIndex = null;")
                && APP_JS.contains("updateScenarioHighlight();"),
            "manual action changes should clear scenario highlight and pending evaluation"
        );
        assert!(
            APP_JS.contains("function applyContinueState(decision)")
                && APP_JS.contains("Fix issue first")
                && APP_JS.contains("I understand the risk")
                && APP_JS.contains("Ready for wallet review"),
            "continue button label/state should be decision-driven through one shared function"
        );
        assert!(
            APP_JS.contains("form.addEventListener('input', handleManualFieldInput);")
                && APP_JS.contains("form.addEventListener('change', handleManualFieldChange);"),
            "manual form edits should use shared delegated invalidation listeners"
        );
        assert!(
            APP_JS.contains("let isProgrammaticUpdate = false;")
                && APP_JS.contains("function withProgrammaticUpdate(fn)"),
            "programmatic scenario/reset updates should be guarded from manual invalidation"
        );
        assert!(
            APP_JS.contains("const nextFieldValue = hasExplicitValue ? text : '';")
                && APP_JS.contains(
                    "const nextRangeValue = hasExplicitValue ? text : swapSlippageRange?.defaultValue || '0';"
                )
                && APP_JS.contains("return nullableNumber(formFields.swap_slippage_percent.value);"),
            "untouched swap slippage should remain empty in the form and serialize as null while slider keeps its visual default"
        );
        assert!(
            APP_JS.contains("SIGN: ['source_network', 'contract_address', 'transaction_origin', 'asset_was_unsolicited'],")
                && APP_JS.contains("asset_was_unsolicited: isFieldVisible(selectedAction, 'asset_was_unsolicited')")
                && APP_JS.contains("setFieldVisibility(name, isFieldVisible(action, name));"),
            "asset_was_unsolicited should be visible only when SIGN is selected"
        );
        assert!(
            APP_JS.contains("new AbortController()")
                && APP_JS.contains("signal: controller.signal")
                && APP_JS.contains("if (error && error.name === 'AbortError')")
                && APP_JS.contains("checkButton.disabled = true;")
                && APP_JS.contains("checkButton.disabled = false;"),
            "evaluation requests should support abort and suppress abort errors"
        );
        assert!(
            INDEX_HTML.contains("Choose a demo scenario or enter transaction details to begin."),
            "default result guidance should match reset state copy"
        );
        assert!(
            INDEX_HTML.contains("<section id=\"result\" class=\"card\" aria-live=\"polite\">"),
            "result region should announce decision changes for assistive technologies"
        );
        assert!(
            {
                let compact = css_without_whitespace(STYLES_CSS);
                compact.contains("@media(max-width:700px){")
                    && compact.contains(
                        ".safety-key-bar{position:static;top:auto;z-index:auto;box-shadow:none",
                    )
            },
            "mobile layout should disable sticky safety key bar to avoid overlap"
        );
        assert!(
            APP_JS.contains("function decisionSummary(decision)")
                && APP_JS.contains("Do not continue until this issue is corrected.")
                && APP_JS.contains("A risk needs your attention before you continue.")
                && APP_JS.contains(
                    "Details match the stated intent. Review your wallet before continuing."
                )
                && APP_JS.contains("Deterministic Rust rules")
                && APP_JS.contains("No custody")
                && APP_JS.contains("No transaction sent"),
            "decision card should include summary text and trust indicators"
        );
        assert!(
            APP_JS
                .matches("resetButton.addEventListener('click', resetExperience)")
                .count()
                == 1,
            "reset should have exactly one click handler"
        );
        assert!(
            APP_JS.contains("function resetExperience()")
                && APP_JS.contains("clearPendingEvaluation();")
                && APP_JS.contains("HTMLFormElement.prototype.reset.call(form);")
                && APP_JS.contains("clearAllIntentFieldValues();"),
            "reset should invoke native form reset before clearing all fields"
        );
        assert!(
            APP_JS.contains("setActionState('SEND', { clearIrrelevant: false, focus: false });"),
            "reset should restore SEND action tab"
        );
        assert!(
            APP_JS.contains("selectedScenarioIndex = null;")
                && APP_JS.contains("updateScenarioHighlight();"),
            "reset should clear selected scenario highlight state"
        );
    }
}
