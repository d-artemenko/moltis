# Feedback Loop

Owner: Moltis core team
Last reviewed: 2026-03-13

Use a tight loop the runtime can falsify:

`intent -> implement -> local validation -> targeted runtime smoke -> capture verdict -> record debt`

## What This Means In Practice

- Run the required mechanical gates in [quality-gates.md](quality-gates.md).
- Use `./scripts/local-validate.sh` for broader local proof whenever the touched flow crosses multiple surfaces.
- Run `just ui-e2e` for web UI behavior changes.
- Record the strongest verdict available instead of translating blocked runtime proof into success wording.

## See Also

- Harness index: [README.md](README.md)
- Quality gates: [quality-gates.md](quality-gates.md)
- Merge playbook: [merge-playbook.md](merge-playbook.md)
- Autonomy levels: [autonomy-levels.md](autonomy-levels.md)
