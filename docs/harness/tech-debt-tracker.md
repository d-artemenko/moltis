# Tech Debt Tracker

Owner: Moltis core team
Last reviewed: 2026-03-13

## Entries

| ID | Area | Issue | Owner | Opened | Last Touched | Next Action | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| MOLTIS-001 | Harness | PR CI mainly checks that local status contexts already exist instead of producing independent runtime proof for the PR. | Core | 2026-03-13 | 2026-03-13 | Add artifact-based runtime proof requirements for runtime-critical PR scopes. | Open |
| MOLTIS-002 | Harness | `scripts/local-validate.sh` does not emit a standardized JSON verdict or durable runtime artifact bundle. | Core | 2026-03-13 | 2026-03-13 | Teach local validation to write machine-readable verdicts and attach failure artifacts. | Open |
| MOLTIS-003 | Runtime proof | Channel/provider/OAuth live diagnostics are spread across docs, e2e, and local scripts instead of one explicit runtime-harness surface. | Core | 2026-03-13 | 2026-03-13 | Add dedicated diagnostics/check commands for live provider/channel proof. | Open |

## See Also

- Harness index: [README.md](README.md)
- Docs freshness: [docs-freshness-policy.md](docs-freshness-policy.md)
