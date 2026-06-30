use sendsure_rust::{demo_scenarios, evaluate, parse_http_request, Decision, Intent, Registries};
use std::io::Write;
use std::net::{TcpListener, TcpStream};

fn main() {
    if std::env::args().any(|a| a == "serve") {
        if let Err(error) = serve("127.0.0.1:8080") {
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
        handle_client(stream?)?;
    }
    Ok(())
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

const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>SendSure</title>
  <link rel="stylesheet" href="/styles.css">
</head>
<body>
  <main>
    <h1>SendSure</h1>
    <p>Wallet-agnostic deterministic transaction preflight safety.</p>

    <div class="actions" role="tablist" aria-label="Intent action type">
      <button type="button" data-action="SEND" class="active">SEND</button>
      <button type="button" data-action="SWAP">SWAP</button>
      <button type="button" data-action="APPROVE">APPROVE</button>
      <button type="button" data-action="SIGN">SIGN</button>
    </div>

    <button type="button" disabled>Wallet adapters — next phase</button>
    <button type="button" id="check">Run preflight check</button>

    <h2>Demo scenarios</h2>
    <div id="scenarios"></div>

    <h2>Intent form</h2>
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
      <input name="entered_destination_tag_or_memo" placeholder="Entered tag or memo">
      <input name="expected_destination_tag_or_memo" placeholder="Expected tag or memo">
      <input name="contract_address" placeholder="Contract address">
      <input name="approval_amount_or_scope" placeholder="Approval amount or scope">
      <input name="swap_slippage_percent" type="number" step="0.1" placeholder="Swap slippage %">
      <input name="transaction_origin" placeholder="Transaction origin">
      <label><input name="asset_was_unsolicited" type="checkbox"> Asset was unsolicited</label>
      <div class="form-buttons">
        <button type="submit" id="evaluate">Evaluate intent</button>
        <button type="button" id="reset">Reset</button>
      </div>
    </form>

    <section id="result" class="card">Choose a demo scenario or enter transaction details to begin.</section>
    <button type="button" id="continue" disabled>Continue</button>

    <p class="note">Deterministic Rust rules-not an LLM-make the decision. SendSure does not request seed phrases or private keys and cannot block actions performed outside this application.</p>
  </main>
  <script src="/app.js"></script>
</body>
</html>"#;

const STYLES_CSS: &str = r#"body{font-family:system-ui;margin:0;background:#0d1117;color:#f0f6fc}main{max-width:980px;margin:auto;padding:32px}.actions,form,#scenarios{display:grid;gap:10px;grid-template-columns:repeat(auto-fit,minmax(180px,1fr))}.form-buttons{display:flex;gap:10px}.actions button.active{outline:2px solid #58a6ff;outline-offset:1px}button,input,select{padding:12px;border-radius:8px;border:1px solid #30363d}button{background:#238636;color:white;cursor:pointer}button:disabled{background:#30363d;cursor:not-allowed}.card{margin:24px 0;padding:24px;border-radius:16px;background:#161b22;border:1px solid #30363d}.STOP{border-color:#f85149}.REVIEW{border-color:#d29922}.READY{border-color:#3fb950}.note{color:#8b949e}.scenario-selected{outline:2px solid #58a6ff;outline-offset:1px}.is-hidden{display:none!important}"#;

const APP_JS: &str = r#"const result=document.getElementById('result');const cont=document.getElementById('continue');const form=document.getElementById('intent-form');const evaluateButton=document.getElementById('evaluate');const resetButton=document.getElementById('reset');const scenarioBox=document.getElementById('scenarios');const checkButton=document.getElementById('check');const actionButtons=[...document.querySelectorAll('.actions button')];const evaluateDefaultLabel=(evaluateButton.textContent||'Evaluate intent').trim();const resultDefault='Choose a demo scenario or enter transaction details to begin.';const evaluatingText='Evaluating with deterministic Rust rules...';const fieldNames=['action_type','source_network','destination_network','asset_symbol','asset_identifier','destination_address','expected_destination_address','entered_destination_tag_or_memo','expected_destination_tag_or_memo','contract_address','approval_amount_or_scope','swap_slippage_percent','transaction_origin'];const allIntentFields=[...fieldNames,'asset_was_unsolicited'];const fieldVisibility={SEND:['source_network','destination_network','asset_symbol','asset_identifier','destination_address','expected_destination_address','entered_destination_tag_or_memo','expected_destination_tag_or_memo','transaction_origin','asset_was_unsolicited'],SWAP:['source_network','destination_network','asset_symbol','asset_identifier','destination_address','expected_destination_address','swap_slippage_percent','transaction_origin'],APPROVE:['source_network','asset_symbol','asset_identifier','contract_address','approval_amount_or_scope','transaction_origin'],SIGN:['source_network','contract_address','transaction_origin','asset_was_unsolicited']};const formFields=Object.fromEntries([...form.elements].filter(el=>el.name).map(el=>[el.name,el]));let selectedScenarioIndex=null;let scenarioButtons=[];let selectedAction='SEND';let activeRequestController=null;let requestGeneration=0;function blank(v){return v==null?'':String(v)}function normalizeAction(action){const next=blank(action).toUpperCase();return fieldVisibility[next]?next:'SEND'}function nullableText(value){if(value==null)return null;const trimmed=String(value).trim();return trimmed===''?null:trimmed}function nullableNumber(value){const text=nullableText(value);if(text==null)return null;const parsed=Number(text);return Number.isFinite(parsed)?parsed:null}function fieldContainer(field){if(!field)return null;return field.type==='checkbox'&&field.parentElement?field.parentElement:field}function clearFieldValue(name){const field=formFields[name];if(!field)return;if(field.type==='checkbox'){field.checked=false;return}if(name!=='action_type'){field.value=''}}function setFieldVisibility(name,visible){const field=formFields[name];if(!field)return;const container=fieldContainer(field);if(container){container.hidden=!visible;container.classList.toggle('is-hidden',!visible)}}function isFieldVisible(action,name){return (fieldVisibility[action]||fieldVisibility.SEND).includes(name)}function focusFirstVisibleField(action){const order=(fieldVisibility[action]||fieldVisibility.SEND).slice();const firstName=order.find(name=>{if(name==='action_type')return false;const field=formFields[name];if(!field)return false;const container=fieldContainer(field);return !(container&&container.hidden)});const firstField=firstName?formFields[firstName]:formFields.source_network;if(firstField&&typeof firstField.focus==='function'){firstField.focus()}}function updateFieldVisibility(action){allIntentFields.forEach(name=>{if(name==='action_type')return;setFieldVisibility(name,isFieldVisible(action,name))})}function clearIrrelevantFieldValues(action){allIntentFields.forEach(name=>{if(name==='action_type')return;if(!isFieldVisible(action,name)){clearFieldValue(name)}})}function setActionState(action,{clearIrrelevant=false,focus=false}={}){selectedAction=normalizeAction(action);if(clearIrrelevant){clearIrrelevantFieldValues(selectedAction)}actionButtons.forEach(button=>{button.classList.toggle('active',button.dataset.action===selectedAction)});formFields.action_type.value=selectedAction;updateFieldVisibility(selectedAction);if(focus){focusFirstVisibleField(selectedAction)}}function updateScenarioHighlight(){scenarioButtons.forEach((button,index)=>{button.classList.toggle('scenario-selected',index===selectedScenarioIndex)})}function clearUiMessages(){document.querySelectorAll('[data-validation-message],.validation-message,.api-error,.error-message').forEach(el=>{el.textContent='';el.classList.remove('show');el.hidden=true});document.querySelectorAll('[data-risk-ack],[data-wallet-handoff],[data-completion-message],#risk-acknowledgment,#wallet-handoff,#completion-message').forEach(el=>{if('checked'in el){el.checked=false}el.textContent='';el.classList.remove('show');el.hidden=true})}function closeOpenModals(){document.querySelectorAll('dialog[open]').forEach(dialog=>{if(typeof dialog.close==='function'){dialog.close()}else{dialog.removeAttribute('open')}});document.querySelectorAll('.modal.open,.panel.open,[data-open="true"]').forEach(el=>{el.classList.remove('open');el.setAttribute('data-open','false');el.hidden=true})}function clearPendingEvaluation(){requestGeneration+=1;if(activeRequestController){activeRequestController.abort();activeRequestController=null}}function populateScenario(intent){fieldNames.forEach(name=>{const field=formFields[name];if(!field)return;field.value=blank(intent?.[name])});formFields.asset_was_unsolicited.checked=Boolean(intent?.asset_was_unsolicited)}function valueForField(name){if(!isFieldVisible(selectedAction,name)){return null}if(name==='swap_slippage_percent'){return nullableNumber(formFields.swap_slippage_percent.value)}return nullableText(formFields[name].value)}function buildIntentFromForm(){return{action_type:selectedAction,source_network:valueForField('source_network'),destination_network:valueForField('destination_network'),asset_symbol:valueForField('asset_symbol'),asset_identifier:valueForField('asset_identifier'),destination_address:valueForField('destination_address'),expected_destination_address:valueForField('expected_destination_address'),entered_destination_tag_or_memo:valueForField('entered_destination_tag_or_memo'),expected_destination_tag_or_memo:valueForField('expected_destination_tag_or_memo'),contract_address:valueForField('contract_address'),approval_amount_or_scope:valueForField('approval_amount_or_scope'),swap_slippage_percent:valueForField('swap_slippage_percent'),transaction_origin:valueForField('transaction_origin'),asset_was_unsolicited:isFieldVisible(selectedAction,'asset_was_unsolicited')?Boolean(formFields.asset_was_unsolicited.checked):false}}function setResultIdle(){result.className='card';result.textContent=resultDefault;cont.disabled=true;cont.textContent='Continue'}function setResultLoading(){result.className='card';result.textContent=evaluatingText;evaluateButton.disabled=true;evaluateButton.textContent='Evaluating...';cont.disabled=true;cont.textContent='Continue'}function continueLabel(decision){if(decision==='READY')return 'Continue';if(decision==='REVIEW')return 'Continue with review';return 'Continue'}function renderResult(payload){result.className='card '+payload.decision;result.innerHTML=`<h2>${payload.decision}</h2><p><b>Rule:</b> ${payload.triggered_rule_id}</p><p>${payload.explanation}</p><p><b>Recommended next step:</b> ${payload.recommended_next_step}</p>`;cont.disabled=payload.decision==='STOP';cont.textContent=continueLabel(payload.decision);evaluateButton.disabled=false;evaluateButton.textContent=evaluateDefaultLabel;result.scrollIntoView({behavior:'smooth',block:'start'})}function renderError(message){result.className='card';result.innerHTML=`<h2>Error</h2><p>${message}</p>`;cont.disabled=true;cont.textContent='Continue';evaluateButton.disabled=false;evaluateButton.textContent=evaluateDefaultLabel;result.scrollIntoView({behavior:'smooth',block:'start'})}async function evaluateIntent(intent){clearPendingEvaluation();const myGeneration=requestGeneration;const controller=new AbortController();activeRequestController=controller;setResultLoading();try{const response=await fetch('/api/evaluate',{method:'POST',cache:'no-store',signal:controller.signal,headers:{'Content-Type':'application/json'},body:JSON.stringify(intent)});if(myGeneration!==requestGeneration){return}if(!response.ok){throw new Error(`Request failed with status ${response.status}`)}const payload=await response.json();if(myGeneration!==requestGeneration){return}renderResult(payload)}catch(error){if(error&&error.name==='AbortError'){return}if(myGeneration!==requestGeneration){return}renderError(`Unable to evaluate intent right now. Please try again in a moment. (${error.message})`)}finally{if(activeRequestController===controller){activeRequestController=null}}}async function evaluateFromForm(){await evaluateIntent(buildIntentFromForm())}function resetExperience(){clearPendingEvaluation();form.reset();setActionState('SEND',{clearIrrelevant:true,focus:false});selectedScenarioIndex=null;updateScenarioHighlight();clearUiMessages();closeOpenModals();setResultIdle();evaluateButton.disabled=false;evaluateButton.textContent=evaluateDefaultLabel;form.scrollIntoView({behavior:'smooth',block:'start'});focusFirstVisibleField('SEND')}actionButtons.forEach(button=>{button.addEventListener('click',()=>{setActionState(button.dataset.action||'SEND',{clearIrrelevant:true,focus:true})})});formFields.action_type.addEventListener('change',()=>{setActionState(formFields.action_type.value,{clearIrrelevant:true,focus:true})});checkButton.addEventListener('click',()=>form.scrollIntoView({behavior:'smooth',block:'start'}));resetButton.addEventListener('click',resetExperience);form.addEventListener('submit',async event=>{event.preventDefault();await evaluateFromForm()});setActionState('SEND');setResultIdle();fetch('/api/scenarios',{cache:'no-store'}).then(response=>response.json()).then(scenarios=>{scenarioBox.innerHTML='';scenarioButtons=[];scenarios.forEach((scenario,index)=>{const button=document.createElement('button');button.type='button';button.textContent=`${index+1}. ${scenario.name}`;button.addEventListener('click',async()=>{selectedScenarioIndex=index;updateScenarioHighlight();setActionState(blank(scenario.intent?.action_type)||'SEND',{clearIrrelevant:false,focus:false});populateScenario(scenario.intent);await evaluateFromForm()});scenarioBox.appendChild(button);scenarioButtons.push(button)})}).catch(()=>{renderError('Unable to load demo scenarios. Refresh and try again.')});"#;
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
            APP_JS.contains("cache:'no-store'"),
            "evaluate fetch should bypass cache"
        );
        assert!(
            APP_JS.contains("button.type='button';"),
            "scenario buttons should be explicitly non-submit"
        );
        assert!(
            APP_JS.contains("SWAP:['source_network','destination_network','asset_symbol','asset_identifier','destination_address','expected_destination_address','swap_slippage_percent','transaction_origin']"),
            "swap action must include destination fields required by engine input"
        );
        assert!(
            APP_JS.contains("formFields.action_type.addEventListener('change'"),
            "action dropdown should share the same action-state handler"
        );
        assert!(
            APP_JS.contains("setActionState(button.dataset.action||'SEND'"),
            "action tabs should use shared action-state handler"
        );
        assert!(
            APP_JS.contains("new AbortController()")
                && APP_JS.contains("signal:controller.signal")
                && APP_JS.contains("if(error&&error.name==='AbortError'){return}"),
            "evaluation requests should support abort and suppress abort errors"
        );
        assert!(
            INDEX_HTML.contains("Choose a demo scenario or enter transaction details to begin."),
            "default result guidance should match reset state copy"
        );
        assert!(
            APP_JS
                .matches("resetButton.addEventListener('click',resetExperience)")
                .count()
                == 1,
            "reset should have exactly one click handler"
        );
        assert!(
            APP_JS.contains("function resetExperience(){clearPendingEvaluation();form.reset();"),
            "reset should call form.reset()"
        );
        assert!(
            APP_JS.contains("setActionState('SEND',{clearIrrelevant:true,focus:false});"),
            "reset should restore SEND action tab"
        );
        assert!(
            APP_JS.contains("selectedScenarioIndex=null;updateScenarioHighlight();"),
            "reset should clear selected scenario highlight state"
        );
    }
}
