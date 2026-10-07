# Codex instructions for SendSure

## Product direction
Read docs/agent-safety-roadmap.md before implementation. Extend the deterministic Rust preflight checker toward one-token Stellar/Soroban testnet agent safeguards. Start with #25 and #26. Complete prerequisite acceptance criteria before dependent work.

## Engineering boundaries
- Rust and authoritative policy determine verdicts. AI may explain results but cannot approve execution or override STOP.
- Separate declared intent, decoded transaction, trusted registry/policy and execution outcome.
- Missing evidence must never silently produce READY. Validate action-specific fields server-side.
- Use integer base units for money, checked arithmetic and chain-specific identifiers.
- A frontend button is not an enforcement boundary. Contract controls must govern protected funds.
- Bind approvals to exact request fields, network, contract, agent, nonce, expiry and policy version.
- Keep demo fixtures explicitly separate from live/testnet verification.
- Never request or store seed phrases/private keys; use wallet signing. Do not log full sensitive requests.

## Workflow
Use focused branches and PRs. Read applicable nested instructions. Inspect current files and existing issues before editing. Keep unrelated changes out. Do not close issues on plans alone.
Preserve seven named demo outcomes in explicit demo mode. Correct unsafe cross-network SEND test expectations intentionally and explain the change; do not preserve unsafe behavior for compatibility.
No deployment, mainnet activity or fund movement is authorized by this setup. Testnet deployment is a later task.
Do not claim a formal audit, production readiness, or tests that were not executed.

## Verification
Run cargo fmt --check, cargo test --locked, cargo clippy --locked --all-targets -- -D warnings and cargo run --locked.
Add meaningful regression tests for changed failure modes. HTTP tests must exercise real requests; browser tests must execute interactions rather than merely search source text.
Once contract code exists, add its contract tests and WASM build to CI.
If tooling is unavailable, state the blocker and use recorded CI results only with their exact commit. Require checks before merging.
PR descriptions explain the trigger, resulting behavior, validation and remaining limitations.
