use sendsure_rust::{demo_scenarios, evaluate, Decision, Intent, Registries};
use std::io::{Read, Write};
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
    let mut buffer = [0_u8; 32768];
    let read = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..read]);
    let first = request.lines().next().unwrap_or_default();
    let body = request.split("\r\n\r\n").nth(1).unwrap_or_default();
    let (status, content_type, response) = if first.starts_with("GET /health ") {
        (
            "200 OK",
            "application/json",
            serde_json::json!({"status":"ok"}).to_string(),
        )
    } else if first.starts_with("GET /api/scenarios ") {
        (
            "200 OK",
            "application/json",
            serde_json::to_string(&demo_scenarios()).unwrap_or_else(|_| "[]".to_string()),
        )
    } else if first.starts_with("POST /api/evaluate ") {
        match serde_json::from_str::<Intent>(body) {
            Ok(intent) => (
                "200 OK",
                "application/json",
                serde_json::to_string(&evaluate(&intent, &Registries::default()))
                    .unwrap_or_else(|_| "{}".to_string()),
            ),
            Err(error) => (
                "400 Bad Request",
                "application/json",
                serde_json::json!({"error": error.to_string()}).to_string(),
            ),
        }
    } else if first.starts_with("GET / ") {
        ("200 OK", "text/html; charset=utf-8", INDEX_HTML.to_string())
    } else if first.starts_with("GET /app.js ") {
        ("200 OK", "application/javascript", APP_JS.to_string())
    } else if first.starts_with("GET /styles.css ") {
        ("200 OK", "text/css", STYLES_CSS.to_string())
    } else {
        (
            "404 Not Found",
            "application/json",
            serde_json::json!({"error":"not found"}).to_string(),
        )
    };
    write!(stream, "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}", response.len())
}

const INDEX_HTML: &str = r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>SendSure</title><link rel="stylesheet" href="/styles.css"></head><body><main><h1>SendSure</h1><p>Wallet-agnostic deterministic transaction preflight safety.</p><div class="actions"><button>SEND</button><button>SWAP</button><button>APPROVE</button><button>SIGN</button></div><button disabled>Connect wallet — coming next</button><button id="check">Check without connecting</button><h2>Demo scenarios</h2><div id="scenarios"></div><h2>Intent form</h2><form id="intent-form"><select name="action_type"><option>SEND</option><option>SWAP</option><option>APPROVE</option><option>SIGN</option></select><input name="source_network" placeholder="Source network"><input name="destination_network" placeholder="Destination network"><input name="asset_symbol" placeholder="Asset symbol"><input name="asset_identifier" placeholder="Token or asset identifier"><input name="destination_address" placeholder="Destination address"><input name="expected_destination_address" placeholder="Expected destination address"><input name="entered_destination_tag_or_memo" placeholder="Entered tag or memo"><input name="expected_destination_tag_or_memo" placeholder="Expected tag or memo"><input name="contract_address" placeholder="Contract address"><input name="approval_amount_or_scope" placeholder="Approval amount or scope"><input name="swap_slippage_percent" type="number" step="0.1" placeholder="Swap slippage %"><input name="transaction_origin" placeholder="Transaction origin"><label><input name="asset_was_unsolicited" type="checkbox"> Asset was unsolicited</label><button type="submit">Evaluate intent</button></form><section id="result" class="card">Result appears here.</section><button id="continue">Continue</button><p class="note">Deterministic Rust rules—not an LLM—make the decision. SendSure does not request seed phrases or private keys and cannot block actions performed outside this application.</p></main><script src="/app.js"></script></body></html>"#;
const STYLES_CSS: &str = r#"body{font-family:system-ui;margin:0;background:#0d1117;color:#f0f6fc}main{max-width:980px;margin:auto;padding:32px}.actions,form,#scenarios{display:grid;gap:10px;grid-template-columns:repeat(auto-fit,minmax(180px,1fr))}button,input,select{padding:12px;border-radius:8px;border:1px solid #30363d}button{background:#238636;color:white;cursor:pointer}button:disabled{background:#30363d;cursor:not-allowed}.card{margin:24px 0;padding:24px;border-radius:16px;background:#161b22;border:1px solid #30363d}.STOP{border-color:#f85149}.REVIEW{border-color:#d29922}.READY{border-color:#3fb950}.note{color:#8b949e}"#;
const APP_JS: &str = r#"const result=document.getElementById('result');const cont=document.getElementById('continue');function show(r){result.className='card '+r.decision;result.innerHTML=`<h2>${r.decision}</h2><p><b>Rule:</b> ${r.triggered_rule_id}</p><p>${r.explanation}</p><p><b>Recommended next step:</b> ${r.recommended_next_step}</p>`;cont.disabled=r.decision==='STOP'}async function evalIntent(intent){const res=await fetch('/api/evaluate',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(intent)});show(await res.json())}fetch('/api/scenarios').then(r=>r.json()).then(s=>{const box=document.getElementById('scenarios');s.forEach((x,i)=>{const b=document.createElement('button');b.textContent=`${i+1}. ${x.name}`;b.onclick=()=>evalIntent(x.intent);box.appendChild(b)})});document.getElementById('intent-form').onsubmit=e=>{e.preventDefault();const f=new FormData(e.target);const v=n=>f.get(n)||null;evalIntent({action_type:v('action_type'),source_network:v('source_network'),destination_network:v('destination_network'),asset_symbol:v('asset_symbol'),asset_identifier:v('asset_identifier'),destination_address:v('destination_address'),expected_destination_address:v('expected_destination_address'),entered_destination_tag_or_memo:v('entered_destination_tag_or_memo'),expected_destination_tag_or_memo:v('expected_destination_tag_or_memo'),contract_address:v('contract_address'),approval_amount_or_scope:v('approval_amount_or_scope'),swap_slippage_percent:v('swap_slippage_percent')?Number(v('swap_slippage_percent')):null,transaction_origin:v('transaction_origin'),asset_was_unsolicited:f.has('asset_was_unsolicited')})};document.getElementById('check').onclick=()=>document.getElementById('intent-form').scrollIntoView();"#;
