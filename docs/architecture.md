## Goal
Review the current SendSure codebase and document a safe post-demo modularization plan without changing runtime behavior.

## Owner
Aman

## Scope
Create docs/architecture-review.md describing:

- Current CLI, HTTP server, API, scenario, and frontend flow
- Recommended module boundaries for models, engine, registries, validators, scenarios, server, routes, and tests
- Risks in the current single-file structure
- A migration order that preserves cargo run, cargo run -- serve, current API routes, rule IDs, and all seven demo outcomes
- Suggestions for improving testability and server resilience

## Restrictions
- Documentation only
- Do not modify src/main.rs
- Do not modify frontend behavior
- Do not change Rust rules, API routes, rule IDs, or scenario outcomes
- Do not push directly to main
- Open a pull request from docs/architecture-review
- Do not merge the pull request

## Acceptance criteria
- docs/architecture-review.md exists
- The document includes a proposed future file tree
- The migration plan explicitly protects current demo behavior
- Pull request targets main

## The document should cover:

- The current CLI, demo scenario, Rust evaluation, API, server, and frontend flow
- Recommended module boundaries
- A proposed future project structure in a modular 
- Risks in the current structure
- A safe migration sequence , how the cde will be migrated step by step guide
- Suggestions for better testing and server resilience " which area of code needs more testing checking the current test code and also any things we need to add for server safety.


## Please preserve:

```bash
cargo run
cargo run -- serve
```
- The current API routes
- Existing rule IDs
- All seven demo scenario outcomes

## Please do not:

- Edit src/main.rs
- Edit the frontend
- Change the Rust rules
- Change API routes
- Push directly to main
- Merge the pull request
