# Moltis Harness Playbook

Owner: Moltis core team
Last reviewed: 2026-03-13

## Purpose

Keep Moltis operability claims anchored in reproducible local validation, runtime smoke, and web UI e2e proof rather than informal CI status folklore.

## Contents

- [architecture-guardrails.md](architecture-guardrails.md): Runtime and crate-boundary invariants.
- [quality-gates.md](quality-gates.md): Required local and runtime validation commands.
- [result-validation.md](result-validation.md): `PASS|BLOCKED|FAIL` policy for Moltis claims.
- [autonomy-levels.md](autonomy-levels.md): Escalation thresholds for local vs external-effect changes.
- [merge-playbook.md](merge-playbook.md): Commit and verification discipline.
- [feedback-loop.md](feedback-loop.md): Build/verify/observe loop for gateway work.
- [tech-debt-tracker.md](tech-debt-tracker.md): Open harness and operability risks.
- [docs-freshness-policy.md](docs-freshness-policy.md): Refresh cadence for docs and commands.

## See Also

- Docs index: [../README.md](../README.md)
- Architecture: [../../ARCHITECTURE.md](../../ARCHITECTURE.md)
