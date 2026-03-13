# Merge Playbook

Owner: Moltis core team
Last reviewed: 2026-03-13

## Goal

Ship focused runtime changes with proof that matches the touched user path.

## Practices

- Keep docs, code, and validation command updates together when they describe the same runtime path.
- Include the exact validation commands run in the merge summary.
- Treat missing runtime proof as `BLOCKED` until an explicit follow-up or debt item exists.
- Do not rely only on PR status propagation when the critical claim depends on local validation or e2e proof.

## See Also

- Harness index: [README.md](README.md)
- Tech debt tracker: [tech-debt-tracker.md](tech-debt-tracker.md)
- Quality gates: [quality-gates.md](quality-gates.md)
