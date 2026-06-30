# SendSure Rust MVP

SendSure is a wallet-agnostic, intent-aware transaction preflight safety layer. A user states what they intend to send, swap, approve, or sign; SendSure compares that intent with a proposed transaction draft and returns `READY`, `REVIEW`, or `STOP`.

The wallet remains responsible for custody and signing. SendSure never requests, collects, stores, or processes seed phrases or private keys.

## Current implementation summary

- Deterministic Rust engine with transfer, swap, approval, signature, and security checks.
- In-memory registries that include XRP Ledger, Ethereum, Base, BNB Smart Chain, Stellar, XRP on XRP Ledger, XLM on Stellar, USDC on Ethereum and Base, and ETH on Ethereum and Base.
- XRP deposit tags and memos are resolved from the in-memory deposit registry when the destination matches the Demo Exchange XRP deposit profile.
- Slippage is evaluated with the requested thresholds: 0% through 3% is READY, greater than 3% through 10% is REVIEW, and greater than 10% is STOP.
- Deterministic STOP rules cover empty SEND/SWAP destination addresses, seed-phrase requests, and private-key requests.
- Twenty-one regression tests cover the seven demo scenarios, tag handling, slippage thresholds, unlimited approvals, trusted/unknown contracts, empty addresses, sensitive-key requests, and split HTTP request bodies.

## What is implemented

- Deterministic Rust safety engine for transfers, tokens, swaps, signatures, approvals, and wallet-security education hooks.
- Chain-neutral `Intent` model designed for a future frontend or wallet adapter.
- STOP-over-REVIEW-over-READY decision precedence.
- In-memory registries for networks, tokens, exchanges, deposit profiles, and contracts.
- Seven polished demo scenarios in the required order.
- Unit/integration tests for the core rules and demo summary.
- Lightweight Rust HTTP server with a plain HTML/CSS/JavaScript interface.

## Deterministic safety rules

SendSure uses local Rust rules only. It does not use an LLM, external risk service, blockchain API, internet request, database, paid API, authentication system, smart contract, browser extension, or live wallet connection to make decisions.

Current primary rules include:

| Rule ID | Decision | Purpose |
| --- | --- | --- |
| `TRANSFER_DESTINATION_TAG_MISMATCH` | STOP | Entered destination tag/memo differs from the expected exchange deposit tag/memo. |
| `TRANSFER_MISSING_DESTINATION_TAG` | STOP | A required destination tag/memo is absent. |
| `TRANSFER_EMPTY_DESTINATION_ADDRESS` | STOP | SEND or SWAP requests omit a destination address. |
| `SECURITY_SEED_OR_PRIVATE_KEY_REQUEST` | STOP | The request appears to seek a seed phrase or private key. |
| `TOKEN_UNSUPPORTED_DESTINATION_NETWORK` | STOP | Asset is not supported on the chosen destination network in the registry. |
| `TOKEN_UNKNOWN_FAMILIAR_SYMBOL` | STOP | Unknown token uses a familiar symbol such as USDC. |
| `SIGN_UNEXPECTED_AIRDROP_INTERACTION` | STOP | Signature touches an unsolicited or unexpected airdrop contract. |
| `APPROVAL_MISSING_SCOPE` | STOP | Approval amount/scope is missing, blank, or unknown. |
| `APPROVAL_UNLIMITED_ALLOWANCE` | STOP | Approval grants unlimited/infinite/max/uint256::MAX allowance. |
| `SWAP_MISSING_SLIPPAGE` | REVIEW | Swap is missing slippage tolerance, so risk check is incomplete. |
| `SWAP_SLIPPAGE_REVIEW` | REVIEW | Swap slippage is greater than 3% through 10%. |
| `SWAP_SLIPPAGE_STOP` | STOP | Swap slippage is greater than 10%. |
| `READY_INTENT_MATCH` | READY | No deterministic warning or stop rule fired. |

## Corrected primary XRP scenario

The demo registry includes a fictional deposit profile:

- Exchange: Demo Exchange
- Network: XRP Ledger
- Destination address: `rDemoExchangeAddress`
- Expected destination tag: `482901`
- Entered destination tag in the mismatch scenario: `482109`
- Result: `STOP`
- Rule ID: `TRANSFER_DESTINATION_TAG_MISMATCH`

Explanation displayed by the engine:

> The destination tag does not match the expected deposit tag for this account. Do not send until the deposit details are verified.

A separate missing-tag test validates `TRANSFER_MISSING_DESTINATION_TAG`, and a registry-derived expected tag path verifies the deposit registry is used as the trusted source.

## Demo scenarios

`cargo run` prints these seven scenarios in order:

1. XRP destination tag mismatch → STOP
2. USDC unsupported destination network → STOP
3. Unknown token using a familiar symbol → STOP
4. Unexpected airdrop interaction → STOP
5. Unlimited approval → STOP
6. High-slippage swap at 7% → REVIEW
7. Valid XRP with the correct destination tag → READY

Expected final summary:

```text
STOP: 5
REVIEW: 1
READY: 1
Total scenarios: 7
```

## Intent model

The reusable `Intent` struct supports:

- action type: `SEND`, `SWAP`, `APPROVE`, `SIGN`
- source network
- destination network
- asset symbol
- token or asset identifier
- destination address
- expected destination address
- entered destination tag or memo
- expected destination tag or memo
- contract address
- approval amount or approval scope
- swap slippage
- transaction origin
- whether the asset was unsolicited

## Run locally

```bash
cargo fmt
cargo test
cargo clippy -- -D warnings
cargo run
```

## Web server

Start the lightweight server:

```bash
cargo run -- serve
```

Open <http://127.0.0.1:8080/>.

Routes:

- `GET /health` returns server health JSON.
- `GET /api/scenarios` returns the seven demonstration scenarios.
- `POST /api/evaluate` accepts an `Intent` JSON body and returns a Rust engine evaluation.

Example evaluation request:

```bash
curl -s http://127.0.0.1:8080/api/evaluate \
  -H 'content-type: application/json' \
  -d '{"action_type":"SEND","source_network":"xrpl","destination_network":"xrpl","asset_symbol":"XRP","asset_identifier":"xrp:xrp","destination_address":"rDemoExchangeAddress","expected_destination_address":"rDemoExchangeAddress","entered_destination_tag_or_memo":"482109","expected_destination_tag_or_memo":"482901","contract_address":null,"approval_amount_or_scope":null,"swap_slippage_percent":null,"transaction_origin":"Demo Exchange","asset_was_unsolicited":false}'
```

The browser JavaScript only collects input and displays API responses. It does not calculate or override safety decisions. For `STOP` results, the SendSure Continue button is disabled. SendSure does not claim it can block actions performed outside the application.

## Roadmap / not added yet

- Live wallet connections
- WalletConnect or Reown
- Stellar Wallets Kit
- Xaman SDK
- Live blockchain transactions
- Real funds
- Paid APIs or Alchemy integration
- Authentication or database
- Browser extension
- Smart contracts
- LLM risk decisions
- React or another large frontend framework
