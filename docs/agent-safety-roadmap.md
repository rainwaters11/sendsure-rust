# SendSure agent safeguards roadmap

Reviewed baseline: b92c59d98eefe311f486f00641acd185d847704c, 7 October 2026.

## Current baseline
Single Rust crate with deterministic rules, fictional in-memory registries, CLI demos, embedded browser interface and a blocking HTTP server. No live wallet, broadcasting, agent execution or Soroban enforcement exists. Source contains 77 test functions. Recorded baseline CI passed; local Rust verification was unavailable because cargo was absent.

## Build order
| Order | Issue | Prerequisites | Outcome |
| --- | --- | --- | --- |
| 1 | #25 | None | Required inputs and sufficient evidence before READY |
| 2 | #26 | None | Same-network SEND; no implicit bridges |
| 3 | #27 | #25, #26 | Bounded transport and executable interaction tests |
| 4 | #28 | #25, #26 | Typed requests, policy versions and amount model |
| 5 | #29 | #28 | Deterministic limits and request-bound approvals |
| 6 | #30 | #28, #29; initial threat model from #32 | Soroban contract enforcement |
| 7 | #31 | #27, #30 | Complete testnet wallet and execution flow |
| Parallel | #32 | Begin threat model now; reports after #28; reconciliation after #30 | Evidence and security documentation |

## Existing backlog integration
#11 explicit REVIEW acknowledgment and #14 inline validation complement #27; frontend validation never replaces Rust validation.
#10 explainability and #12 reports feed #32. #7 decision presentation, #8 remediation, #9 guided flow, #13 scenario transition and #15 accessibility remain useful product work.
#21 and #23 describe refactoring that appears substantially implemented by PR #24; reconcile their acceptance criteria before closing. Do not automatically close them.

## Phase II architecture
Preserve a shared deterministic preflight engine. Add typed policy/request types, then a separate Soroban contract crate with a narrow transfer interface. The contract is authoritative for recipient/token restrictions, authorized agents, spending accounting, owner approval, pause, revocation, expiry and replay protection.
Use one testnet token. Define vault custody and owner recovery explicitly. Agent authority must not allow direct bypass of the guard. Preview checks are advisory; on-chain enforcement checks run atomically with transfer.
AI explanations are optional and follow the policy verdict. They cannot change request fields, verdicts or signing authority.

## Release gates
First demonstrate a permitted transfer, policy rejection, exact-transfer owner approval and pause on testnet. Test unauthorized callers, modified approvals, replay, expiry, split-transfer budget exhaustion, failed-transfer rollback and storage TTL behavior.
Require independent security review before real funds. No paid API, cross-chain bridge or mainnet deployment is required for this milestone.

## CI and repository administration
Use one Rust workflow with formatting, locked tests, Clippy across targets and demo execution. Add browser and contract checks as implemented.
Configure main protection to require rust-quality-gates and PR review; block force pushes and deletion. Repository-setting changes require a supported administrative API; this document is a recommendation until verified active.
