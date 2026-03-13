# Docs Freshness Policy

Owner: Moltis core team
Last reviewed: 2026-03-13

## Review Cadence

- Weekly: `docs/harness/*`, `justfile`, and `scripts/local-validate.sh`.
- On auth/provider/channel contract changes: refresh `docs/harness/quality-gates.md` and `docs/harness/result-validation.md`.
- Before merge of web UI changes: ensure the docs still match required `ui-e2e` proof.

## Staleness Signals

- Missing owner or outdated review date.
- Docs that imply CI alone proves operability.
- Validation commands that drift from `justfile` or `scripts/local-validate.sh`.

## Cleanup Rules

- Archive stale execution plans after completion.
- Update docs in the same change set as validation command changes.
- Keep entrypoint docs short and route depth into targeted nodes.

## See Also

- Harness index: [README.md](README.md)
- Tech debt tracker: [tech-debt-tracker.md](tech-debt-tracker.md)
- Execution plans: [../exec-plans/README.md](../exec-plans/README.md)
