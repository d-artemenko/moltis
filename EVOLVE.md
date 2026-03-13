# Moltis EVOLVE

Owner: Moltis core team
Last reviewed: 2026-03-13

## Rules

- Do not claim runtime operability from PR status presence alone; prefer direct proof from `./scripts/local-validate.sh` or the targeted gate.
- Web UI behavior changes require runtime e2e evidence, not only unit or cargo test coverage.
- When validation depends on external provider/channel state, report `BLOCKED` and strengthen the harness instead of softening the claim.
