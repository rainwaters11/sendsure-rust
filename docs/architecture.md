# SendSure Architecture Review

---

## 1. Current System Overview

SendSure is a deterministic transaction preflight safety layer. It accepts a wallet-agnostic `Intent`, runs local Rust rules (no LLM, blockchain API, or external risk service), and returns one of three decisions: `STOP`, `REVIEW`, or `READY`.

The codebase today is split across two source files:

| File | Role |
| --- | --- |
| `src/lib.rs` | Domain models, registries, rule engine, HTTP request parser, demo scenarios |
| `src/main.rs` | CLI entry, HTTP server, embedded frontend (HTML/CSS/JS), server integration tests |

Integration tests live in `tests/engine.rs`. Server  tests live in `src/main.rs`.

---

## 2. Current Flow

### 2.1 CLI demo flow (`cargo run`)

```
main()
  └─ run_demo()
       ├─ Registries::default()
       ├─ demo_scenarios()          
       └─ for each scenario:
            evaluate(intent, registries)
              ├─ security_rules
              ├─ transfer_rules
              ├─ token_swap_rules
              ├─ signature_rules
              ├─ approval_rules
              └─ sort hits by STOP > REVIEW > READY; return top hit
            print decision, rule ID, explanation, next step
       └─ print summary: STOP: 5, REVIEW: 1, READY: 1
```

The CLI path is read-only with respect to the network. It exercises the same `evaluate()` function the HTTP API uses.

### 2.2 HTTP server flow (`cargo run -- serve`)

```
main()
  └─ serve("127.0.0.1:8080")
       └─ TcpListener::bind -? for each connection:
            handle_client(stream)
              ├─ parse_http_request(stream)   // lib.rs
              └─ route by request line prefix:
                   OPTIONS /api/evaluate                    -> 204 + CORS headers
                   GET  /health                             -> {"status":"ok"}
                   GET  /api/scenarios                      -> demo_scenarios() JSON
                   POST /api/evaluate                       -> deserialize Intent -> evaluate() -> JSON
                   GET  /                                   -> embedded INDEX_HTML
                   GET  /app.js                             -> embedded APP_JS
                   GET  /styles.css                         -> embedded STYLES_CSS
                   GET /assets/sendsure-mark.svg            -> embedded logo
                   GET /assets/sendsure-logo-horizontal.svg -> embedded banner
                    
                   *                      -> 404 JSON
```

The server is a single-threaded, blocking TCP loop. Each request is handled synchronously on the accepting thread. Responses include `Access-Control-Allow-Origin: *` for browser access.

### 2.3 API contract

| Method | Route | Request | Response |
| --- | --- | --- | --- |
| `GET` | `/health` | — | `{"status":"ok"}` |
| `GET` | `/api/scenarios` | — | Array of `{name, intent, expected_decision}` |
| `POST` | `/api/evaluate` | `Intent` JSON body | `Evaluation` JSON |
| `OPTIONS` | `/api/evaluate` | — | `204 No Content` + CORS preflight headers |

Invalid JSON on `POST /api/evaluate` returns `400` with `{"error":"..."}`.

### 2.4 Rule engine flow

`evaluate(intent, registries)` collects rule hits from five rule groups, then applies precedence:

1. **Security** :  seed phrase / private key detection in any intent field
2. **Transfer** :  empty destination, address mismatch, XRP tag/memo validation
3. **Token / swap** :  registry lookups, network support, slippage thresholds
4. **Signature** :  unsolicited airdrop / untrusted contract on `SIGN`
5. **Approval** :  missing scope, unlimited allowance detection

If no rule fires, the engine adds `READY_INTENT_MATCH`. Hits are sorted by decision priority (`STOP` > `REVIEW` > `READY`); the highest-priority hit becomes `triggered_rule_id`.

### 2.5 Demo scenarios

These seven scenarios and their decisions must not change during modularization:

| # | Name | Decision | Triggered rule ID |
| --- | --- | --- | --- |
| 1 | XRP destination tag mismatch | STOP | `TRANSFER_DESTINATION_TAG_MISMATCH` |
| 2 | USDC unsupported destination network | STOP | `TOKEN_UNSUPPORTED_DESTINATION_NETWORK` |
| 3 | Unknown token using a familiar symbol | STOP | `TOKEN_UNKNOWN_FAMILIAR_SYMBOL` |
| 4 | Unexpected airdrop interaction | STOP | `SIGN_UNEXPECTED_AIRDROP_INTERACTION` |
| 5 | Unlimited approval | STOP | `APPROVAL_UNLIMITED_ALLOWANCE` |
| 6 | High-slippage swap at 7% | REVIEW | `SWAP_SLIPPAGE_REVIEW` |
| 7 | Valid XRP with the correct destination tag | READY | `READY_INTENT_MATCH` |

Summary: **STOP: 5, REVIEW: 1, READY: 1**. 
It must persist after all the changes.

### 2.6 Frontend flow

The frontend is embedded in `src/main.rs` as three string constants (`INDEX_HTML`, `STYLES_CSS`, `APP_JS`). It is served statically by the HTTP server; no build step is required.

```
Browser loads GET /
  └─ fetches GET /api/scenarios
       └─ renders seven scenario buttons
  └─ user picks scenario OR fills intent form
       └─ POST /api/evaluate with serialized Intent JSON
            └─ renderResult(decision, rule_id, explanation, next_step)
                 └─ applyContinueState(STOP|REVIEW|READY)
```

Key frontend behaviors (validated by tests in `main.rs`):

- Action tabs (`SEND`, `SWAP`, `APPROVE`, `SIGN`) toggle visible form fields
- Manual edits invalidate prior evaluation results
- In-flight requests use `AbortController` to prevent stale UI updates
- Continue button is disabled on `STOP`, enabled with context-specific labels on `REVIEW`/`READY`
- The frontend never computes safety decisions; it only displays API responses

---

## 3. Recommended Module Boundaries

The current `lib.rs` mixes six concerns. The recommended split:

| Module | Responsibility | Current location |
| --- | --- | --- |
| `models` | `Decision`, `ActionType`, `Intent`, `Evaluation`, `RuleHit`, `Scenario` | `lib.rs` |
| `registries` | `Network`, `Token`, `Exchange`, `DepositProfile`, `ContractProfile`, `Registries` and `Default` impl | `lib.rs` |
| `engine` | `evaluate()`, decision precedence, `hit()` helper | `lib.rs`  |
| `rules` | `security_rules`, `transfer_rules`, `token_swap_rules`, `signature_rules`, `approval_rules` | `lib.rs`  |
| `validators` | Normalization helpers: `norm`, `normalize_text`, `canonical_network_id`, `destination_addresses_match`, `is_uint256_max`, etc. | `lib.rs`  |
| `scenarios` | `demo_scenarios()`, scenario builders (`xrp_intent`, `basic`, `scenario`) | `lib.rs`  |
| `http` | `parse_http_request`, `parse_content_length` | `lib.rs` |
| `server` | `serve`, `handle_client`, route dispatch | `main.rs` |
| `frontend` | Embedded HTML/CSS/JS assets (or external static files) | `main.rs`  |
| `cli` | `run_demo()` | `main.rs`  |

`main.rs` should remain a thin entry point that delegates to `cli` and `server` modules.

---

## 4. Proposed Future Project Structure

```
sendsure-rust/
├── Cargo.toml
├── src/
│   ├── models/
│   │   ├── mod.rs
│   │   ├── decision.rs
│   │   ├── intent.rs
│   │   └── evaluation.rs
│   ├── registries/
│   │   ├── mod.rs
│   │   ├── types.rs                 # Network, Token, Exchange, etc.
│   │   └── default.rs               # Registries::default() seed data
│   ├── engine/
│   │   ├── mod.rs                   # evaluate()
│   │   └── precedence.rs            # Decision::priority, hit sorting
│   ├── rules/
│   │   ├── mod.rs
│   │   ├── security.rs
│   │   ├── transfer.rs
│   │   ├── token_swap.rs
│   │   ├── signature.rs
│   │   └── approval.rs
│   ├── validators/
│   │   ├── mod.rs
│   │   ├── network.rs               # canonical_network_id, network_matches
│   │   ├── address.rs               # destination_addresses_match
│   │   └── allowance.rs             # is_uint256_max, parse_hex/decimal
│   ├── scenarios/
│   │   ├── mod.rs
│   │   └── demo.rs                  # demo_scenarios(), builders
│   ├── http/
│   │   ├── mod.rs
│   │   └── parser.rs                # parse_http_request
│   ├── server/
│   │   ├── mod.rs                   # serve()
│   │   ├── router.rs                # route dispatch
│   │   └── cors.rs                  # CORS header constants
│   ├── cli/
│   │   └── mod.rs                   # run_demo()
│   └── frontend/
│       ├── mod.rs
│       ├── index.html               # or keep as include_str! constants initially
│       ├── app.js
│       └── styles.css
├── tests/
│   ├── engine.rs                    # rule/regression tests (unchanged paths)
│   ├── scenarios.rs                 # optional: demo scenario contract tests
│   └── server.rs                    # move main.rs server tests here
└── docs/
    └── architecture.md              # this document
```

**Note:** A flat `src/` tree (without a workspace) is sufficient for the current MVP size. The workspace layout is listed only as a future option if the project grows beyond a single crate.

---

## 5. Risks in the Current Single-File Structure

### 5.1 `lib.rs` concentration risk

- **909 lines** mixing models, registries, five rule groups, validators, HTTP parsing, and demo data.
- Adding a new rule requires navigating unrelated code; merge conflicts are likely on a team.
- Rule IDs and decision logic are not isolated, making accidental behavior drift harder to spot in review.

### 5.2 `main.rs` concentration risk

- **933 lines** including ~630 lines of embedded frontend strings and ~180 lines of frontend contract tests.
- Server routing, static asset serving, and CLI demo logic share one file with the binary entry point.
- Frontend changes require recompiling the entire binary; there is no separation between transport and presentation.

### 5.3 Testability gaps

- Server route handlers are private functions in `main.rs`; only CORS and frontend-string tests exist there.
- `handle_client` is not directly tested for `/health`, `/api/scenarios`, `/api/evaluate` success/error paths, or 404 responses.
- Rule groups cannot be unit-tested in isolation without calling the full `evaluate()` pipeline.

### 5.4 Server resilience gaps

- **Single-threaded blocking I/O:** one slow client blocks all others.
- **No request size limit:** large `Content-Length` values could exhaust memory.
- **No timeout:** hung connections hold the accept loop indefinitely.
- **No graceful shutdown:** `Ctrl+C` drops in-flight requests.
- **Panic on malformed first line:** `request.lines().next().unwrap_or_default()` is safe, but routing uses string prefix matching that is fragile for paths with query strings or trailing slashes.
- **Connection: close only:** no keep-alive; acceptable for demo, not for production load.

### 5.5 Migration risk

- Moving embedded frontend constants can break `include_str!` paths or test assertions that grep `INDEX_HTML` / `APP_JS` content.
- Splitting `Registries::default()` seed data from rule logic must preserve exact registry contents (network aliases, deposit tag `482901`, contract trust flags).
- Re-export changes in `lib.rs` could break the public API used by `tests/engine.rs` and `main.rs`.

---

## 6. Safe Migration Sequence

Each step should pass `cargo test`, `cargo clippy -- -D warnings`, `cargo run`, and `cargo run -- serve` before proceeding. Do not change rule logic, rule IDs, API routes, or scenario outcomes in any step.

### Phase 0 — Baseline lock (before any move)

1. Confirm `cargo test` passes (engine + main.rs frontend/server tests).
2. Run `cargo run` and verify summary: STOP 5, REVIEW 1, READY 1.
3. Run `cargo run -- serve` and manually hit `/health`, `/api/scenarios`, `/api/evaluate`.
4. Add a CI snapshot test (optional) that asserts all seven scenario rule IDs if not already covered.

### Phase 1 — Extract models and registries (lowest risk)

1. Create `src/models/` with `Decision`, `ActionType`, `Intent`, `Evaluation`, `RuleHit`, `Scenario`.
2. Create `src/registries/` with registry types and `Registries::default()`.
3. Re-export everything from `lib.rs` to preserve the public API.
4. **Verify:** `tests/engine.rs` compiles unchanged; all demo scenario tests pass.

### Phase 2 — Extract validators

1. Move normalization and parsing helpers to `src/validators/`.
2. Keep function signatures identical; no logic changes.
3. **Verify:** network alias tests, EVM case-insensitive address tests, uint256 max tests still pass.

### Phase 3 — Extract rule groups

1. Move each `*_rules` function to `src/rules/<name>.rs`.
2. Export a single `apply_all_rules(intent, registries) -> Vec<RuleHit>` or keep individual functions called from `engine/mod.rs`.
3. **Verify:** every rule ID test in `tests/engine.rs` passes; slippage boundary tests (3%, 3.01%, 10%, 10.01%) unchanged.

### Phase 4 — Extract engine and scenarios

1. Move `evaluate()` to `src/engine/mod.rs`.
2. Move `demo_scenarios()` and builders to `src/scenarios/demo.rs`.
3. **Verify:** `demo_scenarios_follow_expected_decisions` and `seven_demo_scenarios_have_required_summary` pass.

### Phase 5 — Extract HTTP parser

1. Move `parse_http_request` to `src/http/parser.rs`.
2. **Verify:** all `parsing_http_request_*` tests in `tests/engine.rs` pass.

### Phase 6 — Extract server and CLI from main.rs

1. Move `run_demo()` to `src/cli/mod.rs`.
2. Move `serve`, `handle_client`, and route table to `src/server/`.
3. Keep `main.rs` as:

```rust
fn main() {
    if std::env::args().any(|a| a == "serve") {
        sendsure_rust::server::serve("127.0.0.1:8080").unwrap_or(/* ... */);
    } else {
        sendsure_rust::cli::run_demo();
    }
}
```

4. Move `#[cfg(test)] mod tests` from `main.rs` to `tests/server.rs` or `src/server/tests.rs`.
5. **Verify:** CORS test, frontend contract tests, and manual server smoke test pass.

### Phase 7 — Extract frontend assets (optional, lowest priority)

1. Move `INDEX_HTML`, `STYLES_CSS`, `APP_JS` to `src/frontend/` as files loaded via `include_str!`.
2. Update frontend contract tests to read from the new paths.
3. **Do not change frontend behavior** — byte-identical output is the goal.
4. **Verify:** browser demo still loads scenarios and evaluates intents correctly.

### Protected invariants checklist (run after every phase)

- [ ] `cargo run` summary: STOP 5, REVIEW 1, READY 1
- [ ] All seven scenario names and decisions unchanged
- [ ] All rule IDs in README table still fire for their test cases
- [ ] API routes: `GET /health`, `GET /api/scenarios`, `POST /api/evaluate`, `OPTIONS /api/evaluate`, `GET /`, `GET /app.js`, `GET /styles.css`
- [ ] `cargo test` green
- [ ] `cargo clippy -- -D warnings` clean

---

## 7. Testing Improvements

### 7.1 Current test coverage (strong areas)

`tests/engine.rs` provides thorough coverage of:

- All seven demo scenarios and summary counts
- XRP tag mismatch, missing tag, registry-derived expected tag
- Slippage boundaries (0%, 3%, 3.01%, 7%, 10%, 10.01%, negative, missing)
- Unlimited approval (keyword, decimal uint256 max, hex variants)
- Security rules (seed phrase, private key)
- Network alias resolution (xrpl, BSC, display names, whitespace)
- Token symbol/identifier mismatch and familiar-symbol lookalike detection
- EVM case-insensitive address matching vs XRPL case-sensitive matching
- HTTP body parsing (split reads, Content-Length variants, incomplete body, pipelined bytes)

`src/main.rs` tests cover:

- CORS preflight for `OPTIONS /api/evaluate`
- Frontend HTML/JS contract (form structure, abort handling, field visibility, reset flow)

### 7.2 Areas needing more tests

| Area | Gap | Suggested test |
| --- | --- | --- |
| **Server routes** | No integration test for `GET /health`, `GET /api/scenarios`, `POST /api/evaluate` success/400, `404` | Add `tests/server.rs` using existing `round_trip` pattern |
| **Server evaluate path** | JSON round-trip not tested end-to-end through HTTP | POST a known intent, assert `triggered_rule_id` in response body |
| **Rule isolation** | Rules only tested through `evaluate()` | Optional unit tests per rule module with minimal intent fixtures |
| **Decision precedence** | `stop_precedence_beats_review` exists; limited REVIEW-vs-READY cases | Add explicit multi-hit tests asserting `rule_hits` ordering |
| **Registries** | Default registry contents not snapshot-tested | Assert network count, deposit tag value, contract trust flags |
| **Frontend API errors** | No server-side test for malformed JSON → 400 | POST invalid body, assert 400 + error JSON |
| **Scenarios API** | No test that `/api/scenarios` returns seven items with `expected_decision` | HTTP round-trip asserting array length and decisions |

### 7.3 Suggested test layout after modularization

```
tests/
├── engine.rs          # rule/regression (existing)
├── server.rs          # HTTP route integration (new)
└── scenarios.rs       # demo contract: names, order, decisions (optional extract)
```

---

## 8. Server Resilience Improvements

These are post-modularization enhancements. None are required for the demo, but the server module split makes them easier to add.

| Improvement | Why | Suggested approach |
| --- | --- | --- |
| **Thread pool or async runtime** | Single-threaded loop blocks on slow clients | Use `std::thread::spawn` per connection initially; consider `tokio` + `hyper` later |
| **Request body size cap** | Unbounded `Content-Length` reads into memory | Reject bodies over e.g. 64 KB in `parse_http_request` |
| **Read/write timeouts** | Hung clients hold connections forever | Set `TcpStream::set_read_timeout` / `set_write_timeout` |
| **Graceful shutdown** | Ctrl+C drops active requests | Handle `SIGINT`/`SIGTERM`; stop accepting; drain existing connections |
| **Structured routing** | Prefix matching is fragile | Replace `starts_with("GET /health ")` with a small route table or path parser |
| **Health check depth** | `/health` always returns ok | Optionally include engine version or scenario count for ops visibility |
| **Error logging** | Server errors only go to stderr on bind failure | Log route, status, and parse errors without leaking intent contents |
| **Rate limiting** | Demo has no abuse protection | Optional per-IP limit if exposed beyond localhost |

For the demo phase, binding to `127.0.0.1:8080` limits exposure. Any change from localhost should add body limits and timeouts first.

---

## 9. Complete Rule ID Reference

All rule IDs that must be preserved:

| Rule ID | Decision |
| --- | --- |
| `SECURITY_SEED_OR_PRIVATE_KEY_REQUEST` | STOP |
| `TRANSFER_EMPTY_DESTINATION_ADDRESS` | STOP |
| `TRANSFER_DESTINATION_ADDRESS_MISMATCH` | STOP |
| `TRANSFER_MISSING_DESTINATION_TAG` | STOP |
| `TRANSFER_DESTINATION_TAG_MISMATCH` | STOP |
| `TOKEN_ASSET_SYMBOL_MISMATCH` | STOP |
| `TOKEN_UNKNOWN_SOURCE_NETWORK` | STOP |
| `TOKEN_UNSUPPORTED_SOURCE_NETWORK` | STOP |
| `TOKEN_UNKNOWN_DESTINATION_NETWORK` | STOP |
| `TOKEN_UNSUPPORTED_DESTINATION_NETWORK` | STOP |
| `TOKEN_UNKNOWN_FAMILIAR_SYMBOL` | STOP |
| `TOKEN_MISSING_ASSET_IDENTIFIER` | STOP |
| `SWAP_INVALID_SLIPPAGE` | STOP |
| `SWAP_SLIPPAGE_STOP` | STOP |
| `SIGN_UNEXPECTED_AIRDROP_INTERACTION` | STOP |
| `APPROVAL_MISSING_SCOPE` | STOP |
| `APPROVAL_UNLIMITED_ALLOWANCE` | STOP |
| `SWAP_MISSING_SLIPPAGE` | REVIEW |
| `SWAP_SLIPPAGE_REVIEW` | REVIEW |
| `READY_INTENT_MATCH` | READY |

---

## 10. Summary

The SendSure MVP is functionally complete: a deterministic rule engine, seven demo scenarios, a CLI runner, and a self-contained HTTP server with embedded frontend. The main technical debt is organizational — two large files combine unrelated concerns, which increases merge conflict risk and makes server hardening harder.

The recommended path is a **incremental extract-and-re-export migration** across seven phases, with the demo scenario contract and public API re-exports as the primary guardrails. Server resilience and deeper route testing should follow once the server lives in its own module.

**Must preserve throughout:**

```bash
cargo run
cargo run -- serve
```

- Current API routes
- Existing rule IDs
- All seven demo scenario outcomes (STOP 5, REVIEW 1, READY 1)
