# Architecture Guardrails

Owner: Moltis core team
Last reviewed: 2026-03-13

## Intent

Keep Moltis legible as a gateway/runtime system with explicit ownership boundaries between auth, providers, channels, tools, and the web UI.

## Hard Guardrails

- Preserve crate-level ownership: gateway/auth/channel/provider changes stay within their documented boundaries.
- Treat channel/provider/auth flows as runtime contracts, not purely compile-time abstractions.
- Any web UI behavior change needs corresponding e2e coverage or an explicit reason it is server-only.
- Keep runtime proof separate from CI status proxy checks.

## Review Questions

- Does the change keep the runtime boundary clear between crates and external dependencies?
- Is there direct user-path proof for the touched flow, not only a compile/test proxy?
- Would the current `just ui-e2e` and `./scripts/local-validate.sh` still prove the intended contract?

## See Also

- Harness index: [README.md](README.md)
- Architecture: [../../ARCHITECTURE.md](../../ARCHITECTURE.md)
- Quality gates: [quality-gates.md](quality-gates.md)
