use sendsure_rust::{demo_scenarios, evaluate, parse_http_request, Decision, Intent, Registries};
use std::io::Write;
use std::io::{Error, ErrorKind};
use std::net::{TcpListener, TcpStream};

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

fn serve(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("SendSure server listening on http://{addr}");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(error) = handle_client(stream) {
                    if is_ignorable_connection_error(&error) {
                        eprintln!("client connection error (ignored): {error}");
                    } else {
                        eprintln!("client connection error: {error}");
                    }
                }
            }
            Err(error) => {
                if is_ignorable_connection_error(&error) {
                    eprintln!("connection accept error (ignored): {error}");
                } else {
                    eprintln!("connection accept error: {error}");
                }
            }
        }
    }
    Ok(())
}

fn is_ignorable_connection_error(error: &Error) -> bool {
    matches!(
        error.kind(),
        ErrorKind::BrokenPipe
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::UnexpectedEof
    )
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let (request, body) = parse_http_request(&mut stream)?;
    let first = request.lines().next().unwrap_or_default();
    let (status, content_type, response, extra_headers) = if first
        .starts_with("OPTIONS /api/evaluate ")
    {
        (
            "204 No Content",
            "text/plain; charset=utf-8",
            String::new(),
            "Access-Control-Allow-Methods: POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n",
        )
    } else if first.starts_with("GET /health ") {
        (
            "200 OK",
            "application/json",
            serde_json::json!({"status":"ok"}).to_string(),
            "",
        )
    } else if first.starts_with("GET /api/scenarios ") {
        (
            "200 OK",
            "application/json",
            serde_json::to_string(&demo_scenarios()).unwrap_or_else(|_| "[]".to_string()),
            "",
        )
    } else if first.starts_with("POST /api/evaluate ") {
        match serde_json::from_str::<Intent>(&body) {
            Ok(intent) => (
                "200 OK",
                "application/json",
                serde_json::to_string(&evaluate(&intent, &Registries::default()))
                    .unwrap_or_else(|_| "{}".to_string()),
                "",
            ),
            Err(error) => (
                "400 Bad Request",
                "application/json",
                serde_json::json!({"error": error.to_string()}).to_string(),
                "",
            ),
        }
    } else if first.starts_with("GET / ") {
        (
            "200 OK",
            "text/html; charset=utf-8",
            INDEX_HTML.to_string(),
            "",
        )
    } else if first.starts_with("GET /app.js ") {
        ("200 OK", "application/javascript", APP_JS.to_string(), "")
    } else if first.starts_with("GET /styles.css ") {
        ("200 OK", "text/css", STYLES_CSS.to_string(), "")
    } else if first.starts_with("GET /favicon.svg ")
        || first.starts_with("GET /assets/sendsure-mark.svg ")
    {
        (
            "200 OK",
            "image/svg+xml",
            MARK_SVG.to_string(),
            "Cache-Control: public, max-age=86400\r\n",
        )
    } else if first.starts_with("GET /assets/sendsure-logo-horizontal.svg ") {
        (
            "200 OK",
            "image/svg+xml",
            LOGO_HORIZONTAL_SVG.to_string(),
            "",
        )
    } else {
        (
            "404 Not Found",
            "application/json",
            serde_json::json!({"error":"not found"}).to_string(),
            "",
        )
    };
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nAccess-Control-Allow-Origin: *\r\n{extra_headers}Content-Length: {}\r\nConnection: close\r\n\r\n{response}",
        response.len()
    )
}

const LOGO_HORIZONTAL_SVG: &str = include_str!("../assets/sendsure-logo-horizontal.svg");
const MARK_SVG: &str = include_str!("../assets/sendsure-mark.svg");

const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <link rel="icon" href="/assets/sendsure-mark.svg" type="image/svg+xml">
  <link rel="apple-touch-icon" href="/assets/sendsure-mark.svg">
  <title>SendSure</title>
  <link rel="stylesheet" href="/styles.css">
</head>
<body>
  <main>
    <header class="site-header">
      <div class="brand-banner">
        <img src="/assets/sendsure-logo-horizontal.svg" alt="SendSure — Preflight safety before crypto actions become permanent." class="brand-logo" width="760" height="180" decoding="async">
      </div>
      <p class="tagline">Wallet-agnostic deterministic transaction preflight safety.</p>
            <p>SendSure helps crypto users catch risky transaction mistakes before they become permanent.</p>
            <p>No wallet connection required for this demo. Click a scenario below to see how SendSure responds.</p>
    </header>

        <section class="card" aria-label="Decision legend">
            <h2>Decision legend</h2>
            <p><strong>STOP</strong> = Do not continue</p>
            <p><strong>REVIEW</strong> = Slow down and check details</p>
            <p><strong>READY</strong> = Looks safe to continue</p>
        </section>

        <section class="card" aria-label="How to use this demo">
            <h2>How to use this demo</h2>
            <p>1. Choose a sample crypto action.</p>
            <p>2. Click a real-world mistake below.</p>
            <p>3. SendSure will return READY, REVIEW, or STOP before anything is sent.</p>
            <p>This demo uses simulated transaction details. Wallet connection is coming next.</p>
        </section>

        <section class="card" aria-label="Why this matters">
            <p><strong>Why this matters:</strong> Crypto actions can be permanent. SendSure gives users one last safety check before they send, swap, approve, or sign.</p>
        </section>

    <div class="actions" role="tablist" aria-label="Intent action type">
      <button type="button" data-action="SEND" class="active">SEND</button>
      <button type="button" data-action="SWAP">SWAP</button>
      <button type="button" data-action="APPROVE">APPROVE</button>
      <button type="button" data-action="SIGN">SIGN</button>
    </div>

    <p>Wallet connection coming next. This demo uses safe sample transactions only.</p>
    <button type="button" id="check">Run preflight check</button>
    <p>Current demo mode: SendSure is using safe sample transactions so you can test the safety engine without connecting a wallet.</p>

    <h2>Try a real-world crypto mistake</h2>
    <p>Recommended first demo: start with #1 to see how SendSure catches a destination tag mistake.</p>
    <div id="scenarios"></div>

    <h2>Transaction details</h2>
    <form id="intent-form">
      <select name="action_type">
        <option>SEND</option>
        <option>SWAP</option>
        <option>APPROVE</option>
        <option>SIGN</option>
      </select>
      <input name="source_network" placeholder="Source network">
      <input name="destination_network" placeholder="Destination network">
      <input name="asset_symbol" placeholder="Asset symbol">
      <input name="asset_identifier" placeholder="Token or asset identifier">
      <input name="destination_address" placeholder="Destination address">
      <input name="expected_destination_address" placeholder="Expected destination address">
            <div class="field-group" data-field-group="destination-tag">
                <input name="entered_destination_tag_or_memo" placeholder="Destination tag entered in wallet">
                <input name="expected_destination_tag_or_memo" placeholder="Expected destination tag from exchange/deposit page">
                <p class="field-help" id="destination-tag-help">For XRP/XLM-style deposits, compare the tag or memo shown by the exchange with the one entered before sending.</p>
            </div>
      <input name="contract_address" placeholder="Contract address">
      <input name="approval_amount_or_scope" placeholder="Approval amount or scope">
            <div class="field-group" data-field-group="swap-slippage">
                <label for="swap-slippage-range">Slippage tolerance</label>
                <input id="swap-slippage-range" type="range" min="0" max="15" step="0.1" value="0">
                <input name="swap_slippage_percent" type="number" step="0.1" min="0" max="15" placeholder="Swap slippage %">
                <p class="field-help" id="swap-slippage-help">Higher slippage gives a swap more room to move, but can increase risk.</p>
            </div>
      <input name="transaction_origin" placeholder="Transaction origin">
            <div class="field-group" data-field-group="asset-was-unsolicited">
                <label><input name="asset_was_unsolicited" type="checkbox"> I did not ask for this token, NFT, or airdrop</label>
                <p class="field-help">Use this for suspicious airdrops or surprise assets asking you to sign.</p>
            </div>
      <div class="form-buttons">
        <button type="submit" id="evaluate">Evaluate intent</button>
        <button type="button" id="reset">Reset</button>
      </div>
    </form>

    <section id="result" class="card" aria-live="polite">Choose a demo scenario or enter transaction details to begin.</section>
    <button type="button" id="continue" disabled>Continue</button>

        <section class="card" aria-label="Coming next">
            <h2>Coming next</h2>
            <p>Wallet integration</p>
            <p>Real transaction previews</p>
            <p>Expanded token and network registries</p>
            <p>Browser extension / wallet API support</p>
        </section>

                <p class="note">Deterministic Rust rules — not an LLM — make the decision. SendSure never requests seed phrases or private keys and cannot block actions performed outside this application.</p>
                <p class="note">Demo only. SendSure does not provide financial advice or replace wallet review.</p>
                <p class="note">Built by Misty Waters with collaboration from Aman Khan for H.E.R. DAO Rust School.</p>
  </main>
  <script src="/app.js"></script>
</body>
</html>"#;

const STYLES_CSS: &str = r#"html{-webkit-text-size-adjust:100%;text-size-adjust:100%}body{font-family:system-ui;margin:0;background:#0d1117;color:#f0f6fc;font-size:16px}main{max-width:980px;margin:auto;padding:32px;overflow-x:hidden}.site-header{margin-bottom:24px}.brand-banner{padding:18px 20px;border-radius:16px;background:linear-gradient(135deg,#101927 0%,#161b22 52%,#12283a 100%);border:1px solid #30363d;overflow:hidden;text-align:center}.brand-logo{display:block;width:min(100%,680px);max-width:100%;height:auto;object-fit:contain;margin:0 auto}.tagline{margin:14px 0 0;color:#c9d1d9;max-width:70ch;line-height:1.5}.actions,form,#scenarios{display:grid;gap:10px;grid-template-columns:repeat(auto-fit,minmax(180px,1fr))}.form-buttons{display:flex;gap:10px}.field-group{display:grid;gap:8px}.field-group label{font-weight:600}.field-group input[type=range]{width:100%}.field-help{margin:0;color:#c9d1d9;line-height:1.45;font-size:.95rem}.actions button.active{outline:2px solid #58a6ff;outline-offset:1px}button,input,select{padding:12px;border-radius:8px;border:1px solid #30363d}button{background:#238636;color:white;cursor:pointer}button:disabled{background:#30363d;cursor:not-allowed}.card{margin:24px 0;padding:24px;border-radius:16px;background:#161b22;border:1px solid #30363d}.STOP{border-color:#f85149}.REVIEW{border-color:#d29922}.READY{border-color:#3fb950}.decision-banner{display:grid;gap:12px}.decision-header{display:flex;justify-content:space-between;align-items:flex-start;gap:12px;flex-wrap:wrap}.decision-title{margin:0;font-size:2rem;line-height:1;font-weight:800;letter-spacing:.02em}.decision-summary{margin:0;color:#c9d1d9}.rule-line{margin:0;display:flex;align-items:center;gap:8px;color:#c9d1d9}.rule-pill{display:inline-block;padding:4px 8px;border-radius:999px;background:#0d1117;border:1px solid #30363d;color:#f0f6fc;font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,Liberation Mono,monospace;font-size:.78rem}.decision-badge{display:inline-block;padding:4px 10px;border-radius:999px;font-size:.78rem;font-weight:700;letter-spacing:.03em;border:1px solid #30363d;background:#0d1117;color:#f0f6fc}.decision-badge.STOP{border-color:#f85149;color:#f85149}.decision-badge.REVIEW{border-color:#d29922;color:#d29922}.decision-badge.READY{border-color:#3fb950;color:#3fb950}.decision-body p{margin:0 0 10px}.decision-body p:last-child{margin-bottom:0}.trust-row{display:flex;flex-wrap:wrap;gap:8px;margin-top:2px}.trust-chip{display:inline-block;padding:4px 10px;border-radius:999px;border:1px solid #30363d;background:#0d1117;color:#8b949e;font-size:.78rem}.note{color:#8b949e}.scenario-selected{outline:2px solid #58a6ff;outline-offset:1px}.is-hidden{display:none!important}@media(max-width:700px){body{font-size:15px}main{padding:14px;padding-bottom:calc(24px + env(safe-area-inset-bottom))}.site-header{margin-bottom:12px}.brand-banner{padding:10px 12px;border-radius:10px}.brand-logo{width:100%;max-width:100%;height:auto;object-fit:contain}.tagline{font-size:.9rem;line-height:1.35;max-width:100%}h2{font-size:1.4rem;line-height:1.35}p{font-size:.95rem;line-height:1.4}.actions{grid-template-columns:repeat(2,1fr)}#scenarios{grid-template-columns:repeat(auto-fit,minmax(140px,1fr))}form{grid-template-columns:1fr}button{padding:10px 12px;font-size:.95rem;min-height:44px;width:100%;box-sizing:border-box}input,select{padding:9px 10px;font-size:16px;width:100%;box-sizing:border-box;min-height:44px}.form-buttons{flex-wrap:wrap}.form-buttons button{width:auto;flex:1 1 auto}.card{margin:12px 0;padding:14px;border-radius:12px}.decision-title{font-size:1.4rem}}"#;

const APP_JS: &str = r##"
const result = document.getElementById('result');
const cont = document.getElementById('continue');
const form = document.getElementById('intent-form');
const evaluateButton = document.getElementById('evaluate');
const resetButton = document.getElementById('reset');
const scenarioBox = document.getElementById('scenarios');
const checkButton = document.getElementById('check');
const swapSlippageRange = document.getElementById('swap-slippage-range');
const actionButtons = [...document.querySelectorAll('.actions button')];

const evaluateDefaultLabel = (evaluateButton.textContent || 'Evaluate intent').trim();
const resultDefault = 'Choose a demo scenario or enter transaction details to begin.';
const actionChangedText = 'Transaction details changed. Run the preflight check again.';
const evaluatingText = 'Evaluating with deterministic Rust rules...';

const fieldNames = [
    'action_type',
    'source_network',
    'destination_network',
    'asset_symbol',
    'asset_identifier',
    'destination_address',
    'expected_destination_address',
    'entered_destination_tag_or_memo',
    'expected_destination_tag_or_memo',
    'contract_address',
    'approval_amount_or_scope',
    'swap_slippage_percent',
    'transaction_origin',
];

const allIntentFields = [...fieldNames, 'asset_was_unsolicited'];
const fieldVisibility = {
    SEND: [
        'source_network',
        'destination_network',
        'asset_symbol',
        'asset_identifier',
        'destination_address',
        'expected_destination_address',
        'entered_destination_tag_or_memo',
        'expected_destination_tag_or_memo',
        'transaction_origin',
    ],
    SWAP: [
        'source_network',
        'destination_network',
        'asset_symbol',
        'asset_identifier',
        'destination_address',
        'expected_destination_address',
        'swap_slippage_percent',
        'transaction_origin',
    ],
    APPROVE: [
        'source_network',
        'asset_symbol',
        'asset_identifier',
        'contract_address',
        'approval_amount_or_scope',
        'transaction_origin',
    ],
    SIGN: ['source_network', 'contract_address', 'transaction_origin', 'asset_was_unsolicited'],
};

const formFields = Object.fromEntries([...form.elements].filter((el) => el.name).map((el) => [el.name, el]));

let selectedScenarioIndex = null;
let scenarioButtons = [];
let selectedAction = 'SEND';
let activeRequestController = null;
let requestGeneration = 0;
let isProgrammaticUpdate = false;

function blank(value) {
    return value == null ? '' : String(value);
}

function normalizeAction(action) {
    const next = blank(action).toUpperCase();
    return fieldVisibility[next] ? next : 'SEND';
}

function nullableText(value) {
    if (value == null) {
        return null;
    }
    const trimmed = String(value).trim();
    return trimmed === '' ? null : trimmed;
}

function nullableNumber(value) {
    const text = nullableText(value);
    if (text == null) {
        return null;
    }
    const parsed = Number(text);
    return Number.isFinite(parsed) ? parsed : null;
}

function withProgrammaticUpdate(fn) {
    const previous = isProgrammaticUpdate;
    isProgrammaticUpdate = true;
    try {
        return fn();
    } finally {
        isProgrammaticUpdate = previous;
    }
}

function fieldContainer(field) {
    if (!field) {
        return null;
    }
    return field.closest('[data-field-group]') || (field.type === 'checkbox' && field.parentElement ? field.parentElement : field);
}

function clearFieldValue(name) {
    const field = formFields[name];
    if (!field) {
        return;
    }
    if (field.type === 'checkbox') {
        field.checked = false;
        return;
    }
    if (name === 'swap_slippage_percent') {
        setSwapSlippageValue('0');
        return;
    }
    if (name !== 'action_type') {
        field.value = '';
    }
}

function setFieldVisibility(name, visible) {
    const field = formFields[name];
    if (!field) {
        return;
    }
    const container = fieldContainer(field);
    if (container) {
        container.hidden = !visible;
        container.classList.toggle('is-hidden', !visible);
    }
}

function setSwapSlippageValue(value) {
    const nextValue = value == null || value === '' ? '0' : String(value);
    if (formFields.swap_slippage_percent) {
        formFields.swap_slippage_percent.value = nextValue;
    }
    if (swapSlippageRange) {
        swapSlippageRange.value = nextValue;
    }
}

function isFieldVisible(action, name) {
    return (fieldVisibility[action] || fieldVisibility.SEND).includes(name);
}

function focusFirstVisibleField(action) {
    const order = (fieldVisibility[action] || fieldVisibility.SEND).slice();
    const firstName = order.find((name) => {
        if (name === 'action_type') {
            return false;
        }
        const field = formFields[name];
        if (!field) {
            return false;
        }
        const container = fieldContainer(field);
        return !(container && container.hidden);
    });
    const firstField = firstName ? formFields[firstName] : formFields.source_network;
    if (firstField && typeof firstField.focus === 'function') {
        firstField.focus();
    }
}

function updateFieldVisibility(action) {
    allIntentFields.forEach((name) => {
        if (name === 'action_type') {
            return;
        }
        setFieldVisibility(name, isFieldVisible(action, name));
    });
}

function clearIrrelevantFieldValues(action) {
    allIntentFields.forEach((name) => {
        if (name === 'action_type') {
            return;
        }
        if (!isFieldVisible(action, name)) {
            clearFieldValue(name);
        }
    });
}

function setActionState(action, { clearIrrelevant = false, focus = false } = {}) {
    selectedAction = normalizeAction(action);
    if (clearIrrelevant) {
        clearIrrelevantFieldValues(selectedAction);
    }
    actionButtons.forEach((button) => {
        button.classList.toggle('active', button.dataset.action === selectedAction);
    });
    formFields.action_type.value = selectedAction;
    updateFieldVisibility(selectedAction);
    if (focus) {
        focusFirstVisibleField(selectedAction);
    }
}

function updateScenarioHighlight() {
    scenarioButtons.forEach((button, index) => {
        button.classList.toggle('scenario-selected', index === selectedScenarioIndex);
    });
}

function clearUiMessages() {
    document
        .querySelectorAll('[data-validation-message],.validation-message,.api-error,.error-message')
        .forEach((el) => {
            el.textContent = '';
            el.classList.remove('show');
            el.hidden = true;
        });
    document
        .querySelectorAll(
            '[data-risk-ack],[data-wallet-handoff],[data-completion-message],#risk-acknowledgment,#wallet-handoff,#completion-message'
        )
        .forEach((el) => {
            if ('checked' in el) {
                el.checked = false;
            }
            el.textContent = '';
            el.classList.remove('show');
            el.hidden = true;
        });
}

function closeOpenModals() {
    document.querySelectorAll('dialog[open]').forEach((dialog) => {
        if (typeof dialog.close === 'function') {
            dialog.close();
        } else {
            dialog.removeAttribute('open');
        }
    });
    document.querySelectorAll('.modal.open,.panel.open,[data-open="true"]').forEach((el) => {
        el.classList.remove('open');
        el.setAttribute('data-open', 'false');
        el.hidden = true;
    });
}

function clearPendingEvaluation() {
    requestGeneration += 1;
    if (activeRequestController) {
        activeRequestController.abort();
        activeRequestController = null;
    }
}

function invalidateActionEvaluationState() {
    clearPendingEvaluation();
    selectedScenarioIndex = null;
    updateScenarioHighlight();
    clearUiMessages();
    closeOpenModals();
    result.className = 'card';
    result.textContent = actionChangedText;
    applyContinueState();
    evaluateButton.disabled = false;
    checkButton.disabled = false;
    evaluateButton.textContent = evaluateDefaultLabel;
}

function applyManualActionChange(action) {
    withProgrammaticUpdate(() => {
        setActionState(action, { clearIrrelevant: true, focus: true });
    });
    invalidateActionEvaluationState();
}

function shouldInvalidateForManualEdit(target) {
    if (isProgrammaticUpdate || !target || !target.name) {
        return false;
    }
    if (target.name === 'action_type') {
        return false;
    }
    return true;
}

function handleManualFieldInput(event) {
    const target = event.target;
    if (!shouldInvalidateForManualEdit(target)) {
        return;
    }
    const type = (target.type || '').toLowerCase();
    if (type === 'checkbox') {
        return;
    }
    invalidateActionEvaluationState();
}

function handleManualFieldChange(event) {
    const target = event.target;
    if (!shouldInvalidateForManualEdit(target)) {
        return;
    }
    const tag = (target.tagName || '').toUpperCase();
    const type = (target.type || '').toLowerCase();
    const isCheckbox = tag === 'INPUT' && type === 'checkbox';
    const isSelect = tag === 'SELECT';
    if (!isCheckbox && !isSelect) {
        return;
    }
    invalidateActionEvaluationState();
}

function populateScenario(intent, scenarioName) {
    fieldNames.forEach((name) => {
        const field = formFields[name];
        if (!field) {
            return;
        }
        if (name === 'swap_slippage_percent') {
            setSwapSlippageValue(intent?.[name]);
            return;
        }
        if (name === 'expected_destination_tag_or_memo') {
            field.value = scenarioName === 'XRP destination tag mismatch' ? '482901' : blank(intent?.[name]);
            return;
        }
        field.value = blank(intent?.[name]);
    });
    setSwapSlippageValue(intent?.swap_slippage_percent);
    formFields.asset_was_unsolicited.checked = Boolean(intent?.asset_was_unsolicited);
}

function valueForField(name) {
    if (!isFieldVisible(selectedAction, name)) {
        return null;
    }
    if (name === 'swap_slippage_percent') {
        return nullableNumber(formFields.swap_slippage_percent.value);
    }
    return nullableText(formFields[name].value);
}

function buildIntentFromForm() {
    return {
        action_type: selectedAction,
        source_network: valueForField('source_network'),
        destination_network: valueForField('destination_network'),
        asset_symbol: valueForField('asset_symbol'),
        asset_identifier: valueForField('asset_identifier'),
        destination_address: valueForField('destination_address'),
        expected_destination_address: valueForField('expected_destination_address'),
        entered_destination_tag_or_memo: valueForField('entered_destination_tag_or_memo'),
        expected_destination_tag_or_memo: valueForField('expected_destination_tag_or_memo'),
        contract_address: valueForField('contract_address'),
        approval_amount_or_scope: valueForField('approval_amount_or_scope'),
        swap_slippage_percent: valueForField('swap_slippage_percent'),
        transaction_origin: valueForField('transaction_origin'),
        asset_was_unsolicited: isFieldVisible(selectedAction, 'asset_was_unsolicited')
            ? Boolean(formFields.asset_was_unsolicited.checked)
            : false,
    };
}

function applyContinueState(decision) {
    if (decision === 'STOP') {
        cont.disabled = true;
        cont.textContent = 'Fix issue first';
        return;
    }
    if (decision === 'REVIEW') {
        cont.disabled = false;
        cont.textContent = 'I understand the risk';
        return;
    }
    if (decision === 'READY') {
        cont.disabled = false;
        cont.textContent = 'Ready for wallet review';
        return;
    }
    cont.disabled = true;
    cont.textContent = 'Continue';
}

function escapeHtml(value) {
    return blank(value)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
}

function decisionSummary(decision) {
    if (decision === 'READY') {
        return 'Details match the stated intent. Review your wallet before continuing.';
    }
    if (decision === 'REVIEW') {
        return 'A risk needs your attention before you continue.';
    }
    if (decision === 'STOP') {
        return 'Do not continue until this issue is corrected.';
    }
    return '';
}

function clearAllIntentFieldValues() {
    allIntentFields.forEach((name) => {
        if (name === 'action_type') {
            formFields.action_type.value = 'SEND';
            return;
        }
        clearFieldValue(name);
    });
}

function setResultIdle() {
    result.className = 'card';
    result.textContent = resultDefault;
    applyContinueState();
    checkButton.disabled = false;
}

function setResultLoading() {
    result.className = 'card';
    result.textContent = evaluatingText;
    evaluateButton.disabled = true;
    checkButton.disabled = true;
    evaluateButton.textContent = 'Evaluating...';
    applyContinueState();
}

function renderResult(payload) {
    result.className = 'card ' + payload.decision;
    const decision = escapeHtml(payload.decision);
    const summary = escapeHtml(decisionSummary(payload.decision));
    const ruleId = escapeHtml(payload.triggered_rule_id);
    const explanation = escapeHtml(payload.explanation);
    const nextStep = escapeHtml(payload.recommended_next_step);
    result.innerHTML = `<div class="decision-banner"><div class="decision-header"><h2 class="decision-title">${decision}</h2><span class="decision-badge ${decision}">${decision}</span></div><p class="decision-summary">${summary}</p><p class="rule-line"><strong>Rule:</strong> <span class="rule-pill">${ruleId}</span></p><div class="decision-body"><p>${explanation}</p><p><b>Recommended next step:</b> ${nextStep}</p></div><div class="trust-row" aria-label="Trust indicators"><span class="trust-chip">Deterministic Rust rules</span><span class="trust-chip">No custody</span><span class="trust-chip">No transaction sent</span></div></div>`;
    applyContinueState(payload.decision);
    evaluateButton.disabled = false;
    checkButton.disabled = false;
    evaluateButton.textContent = evaluateDefaultLabel;
    result.scrollIntoView({ behavior: 'smooth', block: 'start' });
}

function renderError(message) {
    result.className = 'card';
    result.innerHTML = `<h2>Error</h2><p>${message}</p>`;
    applyContinueState();
    evaluateButton.disabled = false;
    checkButton.disabled = false;
    evaluateButton.textContent = evaluateDefaultLabel;
    result.scrollIntoView({ behavior: 'smooth', block: 'start' });
}

async function evaluateIntent(intent) {
    clearPendingEvaluation();
    const myGeneration = requestGeneration;
    const controller = new AbortController();
    activeRequestController = controller;
    setResultLoading();
    try {
        const response = await fetch('/api/evaluate', {
            method: 'POST',
            cache: 'no-store',
            signal: controller.signal,
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(intent),
        });
        if (myGeneration !== requestGeneration) {
            return;
        }
        if (!response.ok) {
            throw new Error(`Request failed with status ${response.status}`);
        }
        const payload = await response.json();
        if (myGeneration !== requestGeneration) {
            return;
        }
        renderResult(payload);
    } catch (error) {
        if (error && error.name === 'AbortError') {
            return;
        }
        if (myGeneration !== requestGeneration) {
            return;
        }
        renderError(`Unable to evaluate intent right now. Please try again in a moment. (${error.message})`);
    } finally {
        if (activeRequestController === controller) {
            activeRequestController = null;
        }
    }
}

async function evaluateFromForm() {
    await evaluateIntent(buildIntentFromForm());
}

function resetExperience() {
    clearPendingEvaluation();
    withProgrammaticUpdate(() => {
        HTMLFormElement.prototype.reset.call(form);
        clearAllIntentFieldValues();
        setActionState('SEND', { clearIrrelevant: false, focus: false });
    });
    selectedScenarioIndex = null;
    updateScenarioHighlight();
    clearUiMessages();
    closeOpenModals();
    setResultIdle();
    evaluateButton.disabled = false;
    checkButton.disabled = false;
    evaluateButton.textContent = evaluateDefaultLabel;
    form.scrollIntoView({ behavior: 'smooth', block: 'start' });
    focusFirstVisibleField('SEND');
}

actionButtons.forEach((button) => {
    button.addEventListener('click', () => {
        applyManualActionChange(button.dataset.action || 'SEND');
    });
});

formFields.action_type.addEventListener('change', () => {
    applyManualActionChange(formFields.action_type.value);
});

form.addEventListener('input', handleManualFieldInput);
form.addEventListener('change', handleManualFieldChange);

if (swapSlippageRange && formFields.swap_slippage_percent) {
    swapSlippageRange.addEventListener('input', () => {
        withProgrammaticUpdate(() => {
            formFields.swap_slippage_percent.value = swapSlippageRange.value;
        });
        invalidateActionEvaluationState();
    });
    formFields.swap_slippage_percent.addEventListener('input', () => {
        withProgrammaticUpdate(() => {
            swapSlippageRange.value = formFields.swap_slippage_percent.value || '0';
        });
    });
}

checkButton.addEventListener('click', async () => {
    form.scrollIntoView({ behavior: 'smooth', block: 'start' });
    await evaluateFromForm();
});
resetButton.addEventListener('click', resetExperience);

form.addEventListener('submit', async (event) => {
    event.preventDefault();
    await evaluateFromForm();
});

withProgrammaticUpdate(() => {
    setActionState('SEND');
});
setResultIdle();

fetch('/api/scenarios', { cache: 'no-store' })
    .then((response) => response.json())
    .then((scenarios) => {
        scenarioBox.innerHTML = '';
        scenarioButtons = [];
        scenarios.forEach((scenario, index) => {
            const button = document.createElement('button');
            button.type = 'button';
            button.textContent = `${index + 1}. ${scenario.name}`;
            button.addEventListener('click', async () => {
                selectedScenarioIndex = index;
                updateScenarioHighlight();
                withProgrammaticUpdate(() => {
                    setActionState(blank(scenario.intent?.action_type) || 'SEND', {
                        clearIrrelevant: false,
                        focus: false,
                    });
                    populateScenario(scenario.intent, scenario.name);
                });
                await evaluateFromForm();
            });
            scenarioBox.appendChild(button);
            scenarioButtons.push(button);
        });
    })
    .catch(() => {
        renderError('Unable to load demo scenarios. Refresh and try again.');
    });
"##;
#[cfg(test)]
mod tests {
    use super::handle_client;
    use super::{APP_JS, INDEX_HTML};
    use std::io::{Read, Write};
    use std::net::{Shutdown, TcpListener, TcpStream};
    use std::thread;

    fn round_trip(request: &str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral listener");
        let addr = listener.local_addr().expect("read listener address");
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept connection");
            handle_client(stream).expect("serve one client");
        });

        let mut client = TcpStream::connect(addr).expect("connect to test server");
        client.write_all(request.as_bytes()).expect("send request");
        client
            .shutdown(Shutdown::Write)
            .expect("shutdown write side");
        let mut response = String::new();
        client.read_to_string(&mut response).expect("read response");
        server.join().expect("join server thread");
        response
    }

    #[test]
    fn options_evaluate_returns_success_with_required_cors_headers() {
        let response = round_trip(
            "OPTIONS /api/evaluate HTTP/1.1\r\nHost: example\r\nContent-Length: 0\r\n\r\n",
        );
        assert!(response.starts_with("HTTP/1.1 204 No Content\r\n"));
        assert!(response.contains("Access-Control-Allow-Origin: *\r\n"));
        assert!(response.contains("Access-Control-Allow-Methods: POST, OPTIONS\r\n"));
        assert!(response.contains("Access-Control-Allow-Headers: Content-Type\r\n"));
        assert!(response.contains("Content-Length: 0\r\n"));
    }

    #[test]
    fn favicon_routes_serve_shield_mark_svg() {
        for path in ["/favicon.svg", "/assets/sendsure-mark.svg"] {
            let request = format!("GET {path} HTTP/1.1\r\nHost: example\r\n\r\n");
            let response = round_trip(&request);
            assert!(
                response.starts_with("HTTP/1.1 200 OK\r\n"),
                "expected 200 for {path}"
            );
            assert!(
                response.contains("Content-Type: image/svg+xml\r\n"),
                "expected svg content type for {path}"
            );
            assert!(
                response.contains("<svg xmlns=\"http://www.w3.org/2000/svg\""),
                "expected svg body for {path}"
            );
        }
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
