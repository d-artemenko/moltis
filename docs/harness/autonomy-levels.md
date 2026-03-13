# Autonomy Levels

Owner: Moltis core team
Last reviewed: 2026-03-13

## Level 1: Deterministic Local Changes

Proceed autonomously for docs, additive tests, lint/build fixes, and localized refactors.

## Level 2: Contract-Sensitive Changes

State assumptions explicitly for wire protocol, auth, provider, channel, or config-schema changes.

## Level 3: Runtime-Affecting Changes

Pause if the change can alter external provider credentials, channel transport semantics, or deployment/runtime bootstrap behavior.

## Level 4: External-Effect Changes

Do not run autonomously when the action touches production secrets, live service exposure, or destructive persistent data operations.

## See Also

- Harness index: [README.md](README.md)
- Quality gates: [quality-gates.md](quality-gates.md)
- Merge playbook: [merge-playbook.md](merge-playbook.md)
