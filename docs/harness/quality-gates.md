# Moltis Quality Gates

Owner: Moltis core team
Last reviewed: 2026-03-13

## Required Gates

- `format`: `just format-check`
- `web lint`: `biome ci crates/web/src/assets/js/`
- `i18n`: `./scripts/i18n-check.sh`
- `rust lint`: `cargo clippy --workspace --all-features --all-targets -- -D warnings`
- `tests`: `cargo nextest run --all-features --profile ci`
- `ui e2e`: `just ui-e2e`
- `local deep validation`: `./scripts/local-validate.sh`

## Operability Claim Policy

- `PASS`: the required mechanical gates and user-path proof both passed.
- `BLOCKED`: runtime proof depends on unavailable local/remote state outside the changed code.
- `FAIL`: the gate ran and exposed a regression.
- Do not claim channel/provider/auth flows work from PR status propagation alone.
- Use `./scripts/local-validate.sh` when the touched path spans multiple areas or when CI status presence is weaker than direct local proof.

## Merge Criteria

- No warnings/errors in pinned lint/format toolchain.
- Channel/provider/auth changes include runtime smoke or local validation evidence.
- Web UI changes ship with e2e coverage.
- If runtime proof is blocked, record the blocker explicitly in `docs/harness/tech-debt-tracker.md`.
