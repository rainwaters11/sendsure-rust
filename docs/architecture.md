# SendSure Architecture Review

---

## 1. Current System Overview

SendSure is a deterministic transaction preflight safety layer. It accepts a wallet-agnostic `Intent`, runs local Rust rules (no LLM, blockchain API, or external risk service), and returns one of three decisions: `STOP`, `REVIEW`, or `READY`.

The implementation is organized under `src/models/`. The crate root preserves the
public API through re-exports, while the binary remains a thin entry point:

| File | Role |
| --- | --- |
| `src/models/` | Domain types, registries, rules, engine, validators, scenarios, HTTP parser, server, CLI, and embedded frontend |
| `src/lib.rs` | Declares `models` and re-exports the supported public API |
| `src/main.rs` | Selects CLI demo or HTTP server mode and delegates to the library |
| `tests/engine.rs` | Rule-engine, parser, and scenario regression tests |
| `tests/server.rs` | HTTP server and frontend contract integration tests |

Production logic and tests no longer live in `src/lib.rs` or `src/main.rs`.

---

## 2. Current Flow

### 2.1 CLI demo flow (`cargo run`)

```
main()
  └─ models::cli::run_demo()
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
  └─ models::server::serve("0.0.0.0:$PORT")
       └─ TcpListener::bind -? for each connection:
            handle_client(stream)
              ├─ models::http::parse_http_request(stream)
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

These seven scenarios and their decisions are protected compatibility invariants:

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

The frontend assets live in `src/models/frontend/`. `mod.rs` embeds
`index.html`, `styles.css`, `app.js`, and the SVG brand assets with
`include_str!`. They are served statically by the HTTP server; no frontend build
step is required.

```
Browser loads GET /
  └─ fetches GET /api/scenarios
       └─ renders seven scenario buttons
  └─ user picks scenario OR fills intent form
       └─ POST /api/evaluate with serialized Intent JSON
            └─ renderResult(decision, rule_id, explanation, next_step)
                 └─ applyContinueState(STOP|REVIEW|READY)
```

Key frontend behaviors (validated by tests in `tests/server.rs`):

- Action tabs (`SEND`, `SWAP`, `APPROVE`, `SIGN`) toggle visible form fields
- Manual edits invalidate prior evaluation results
- In-flight requests use `AbortController` to prevent stale UI updates
- Continue button is disabled on `STOP`, enabled with context-specific labels on `REVIEW`/`READY`
- The frontend never computes safety decisions; it only displays API responses

---

## 3. Implemented Module Boundaries

| Module | Responsibility | Current location |
| --- | --- | --- |
| Core models | `Decision`, `ActionType`, `Intent`, `Evaluation`, `RuleHit`, `Scenario` | `src/models/*.rs` |
| `registries` | Registry types and `Registries::default()` seed data | `src/models/registries/` |
| `engine` | `evaluate()`, decision precedence, hit construction and sorting | `src/models/engine/` |
| `rules` | Security, transfer, token/swap, signature, and approval rules | `src/models/rules/` |
| `validators` | Network, address, and allowance normalization/validation | `src/models/validators/` |
| `scenarios` | `demo_scenarios()` and scenario builders | `src/models/scenarios/` |
| `http` | HTTP request and `Content-Length` parsing | `src/models/http/` |
| `server` | TCP listener, connection handling, routing, and CORS | `src/models/server/` |
| `frontend` | Embedded HTML, CSS, JavaScript, and SVG assets | `src/models/frontend/` |
| `cli` | `run_demo()` | `src/models/cli/` |

`src/models/mod.rs` owns module declarations and internal/public re-exports.
`src/lib.rs` forwards the supported crate API, so existing callers can continue
using paths such as `sendsure_rust::evaluate` and `sendsure_rust::serve`.
`src/main.rs` is intentionally limited to argument/environment handling and
delegation.

---

## 4. Current Project Structure

```
sendsure-rust/
├── Cargo.toml
├── src/
│   ├── models/
│   │   ├── mod.rs
│   │   ├── decision.rs
│   │   ├── intent.rs
│   │   ├── evaluation.rs
│   │   ├── scenario.rs
│   │   ├── registries/
│   │   │   ├── mod.rs
│   │   │   ├── types.rs             # Network, Token, Exchange, etc.
│   │   │   └── default.rs           # Registries::default() seed data
│   │   ├── engine/
│   │   │   ├── mod.rs               # evaluate()
│   │   │   └── precedence.rs        # Decision::priority, hit sorting
│   │   ├── rules/
│   │   │   ├── mod.rs
│   │   │   ├── security.rs
│   │   │   ├── transfer.rs
│   │   │   ├── token_swap.rs
│   │   │   ├── signature.rs
│   │   │   └── approval.rs
│   │   ├── validators/
│   │   │   ├── mod.rs
│   │   │   ├── network.rs
│   │   │   ├── address.rs
│   │   │   └── allowance.rs
│   │   ├── scenarios/
│   │   │   ├── mod.rs
│   │   │   └── demo.rs
│   │   ├── http/
│   │   │   ├── mod.rs
│   │   │   └── parser.rs
│   │   ├── server/
│   │   │   ├── mod.rs
│   │   │   ├── router.rs
│   │   │   └── cors.rs
│   │   ├── cli/
│   │   │   └── mod.rs
│   │   └── frontend/
│   │       ├── mod.rs
│   │       ├── index.html
│   │       ├── app.js
│   │       └── styles.css
│   ├── lib.rs                       # public re-exports
│   └── main.rs                      # thin binary entry point
├── assets/
│   ├── sendsure-mark.svg
│   └── sendsure-logo-horizontal.svg
├── tests/
│   ├── engine.rs                    # rule/parser/scenario regression tests
│   └── server.rs                    # server and frontend integration tests
└── docs/
    └── architecture.md              # this document
```

The project remains a single crate. Modules are nested under `src/models/` by
project convention; this is the implemented layout, not a future proposal.

---

## 5. Current Risks and Testability Gaps

### 5.1 Module organization

- The former `lib.rs` and `main.rs` concentration risks have been resolved.
- All application modules are nested under `src/models/` by project convention.
  This is consistent internally, although transport and presentation modules
  (`server`, `cli`, and `frontend`) are not domain models in the strict sense.
- `src/lib.rs` re-exports the supported API. Changes to these re-exports can
  still break external callers and integration tests.

### 5.2 Testability gaps

- `tests/server.rs` exercises CORS, connection-error handling, favicon routes,
  and frontend asset contracts through a hidden `test_support` API.
- End-to-end coverage is still missing for `/health`, `/api/scenarios`,
  `/api/evaluate` success and malformed JSON responses, and 404 responses.
- Rule groups are tested thoroughly through `evaluate()`, but not independently
  as isolated modules.

### 5.3 Server resilience gaps

- **Single-threaded blocking I/O:** one slow client blocks all others.
- **No request size limit:** large `Content-Length` values could exhaust memory.
- **No timeout:** hung connections hold the accept loop indefinitely.
- **No graceful shutdown:** `Ctrl+C` drops in-flight requests.
- **Fragile route matching:** string-prefix routing can mishandle paths with query strings or trailing slashes.
- **Connection: close only:** no keep-alive; acceptable for demo, not for production load.

### 5.4 Maintenance risk

- Frontend asset moves can break relative `include_str!` paths.
- Changes to `Registries::default()` must preserve expected aliases, deposit
  profiles, token identifiers, and contract trust flags.
- The hidden `test_support` module exposes wrappers solely so integration tests
  can exercise internal server behavior; it should not become application API.

---

## 6. Modularization Status and Verification

The extract-and-re-export migration is complete:

1. Core models and registries were extracted under `src/models/`.
2. Validators and five rule groups were separated by responsibility.
3. Evaluation precedence and demo scenarios were extracted.
4. HTTP parsing, server routing/CORS, and CLI execution were extracted.
5. Frontend source was moved to standalone embedded asset files.
6. Engine, server, and frontend coverage was consolidated under `tests/`.
7. `src/lib.rs` retained compatibility re-exports and `src/main.rs` became a
   thin entry point.

Use these commands when changing module boundaries or behavior:

```bash
cargo test
cargo clippy -- -D warnings
cargo run
cargo run -- serve
```

Protected invariants:

- `cargo run` summary remains STOP 5, REVIEW 1, READY 1.
- All seven scenario names, decisions, and rule IDs remain unchanged.
- Existing public re-exports continue to compile for `tests/engine.rs`.
- API routes remain available: `GET /health`, `GET /api/scenarios`,
  `POST /api/evaluate`, `OPTIONS /api/evaluate`, `GET /`, `GET /app.js`,
  `GET /styles.css`, `GET /assets/sendsure-mark.svg`, and
  `GET /assets/sendsure-logo-horizontal.svg`.
- `cargo test` and `cargo clippy -- -D warnings` remain clean.

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

`tests/server.rs` covers:

- CORS preflight for `OPTIONS /api/evaluate`
- Ignorable and non-ignorable connection error handling
- SVG favicon/static asset routes
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

### 7.3 Current test layout

```
tests/
├── engine.rs          # rule, parser, and scenario regression tests
└── server.rs          # HTTP server and frontend integration tests
```

Scenario contract tests currently remain in `tests/engine.rs`. They may be
extracted to `tests/scenarios.rs` if that suite grows.

---

## 8. Server Resilience Improvements

These are future enhancements. None are required for the demo, but the current
server module boundary makes them easier to add.

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

The binary currently binds to `0.0.0.0:$PORT` (`8080` by default). Deployments
should therefore add body limits, timeouts, and suitable network-level access
controls before exposing the server to untrusted traffic.

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

The SendSure MVP is functionally complete and modularized under `src/models/`.
`src/lib.rs` provides compatibility re-exports, `src/main.rs` is a thin binary
entry point, frontend assets have standalone source files, and integration tests
live under `tests/`.

The next priorities are deeper route-level integration coverage and server
resilience. Future structural changes must preserve:

```bash
cargo run
cargo run -- serve
cargo test
cargo clippy -- -D warnings
```

- Current API routes
- Existing rule IDs
- All seven demo scenario outcomes (STOP 5, REVIEW 1, READY 1)
