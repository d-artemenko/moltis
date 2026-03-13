# Moltis Result Validation

Owner: Moltis core team
Last reviewed: 2026-03-13

## Purpose

Translate Moltis runtime and UI claims into explicit `PASS`, `BLOCKED`, or `FAIL` outcomes backed by executable proof.

## Required Evidence

- Mechanical baseline: `just format-check`, `./scripts/i18n-check.sh`, `cargo clippy --workspace --all-features --all-targets -- -D warnings`, `cargo nextest run --all-features --profile ci`
- Web UI claims: `just ui-e2e`
- Cross-surface local proof: `./scripts/local-validate.sh`

## Status Rules

- `PASS`: the relevant proof completed successfully.
- `BLOCKED`: the relevant proof requires unavailable runtime state or infrastructure.
- `FAIL`: the proof ran and found a regression.

## Claim Rules

- Do not claim operability from CI status propagation alone.
- Use the strongest available runtime proof for the touched flow.
- When proof is blocked, record the blocker as debt or incident follow-up instead of weakening the wording.
