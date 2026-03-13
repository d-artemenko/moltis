# Moltis Architecture (Workspace Overlay)

Owner: Moltis core team
Last reviewed: 2026-03-02

This overlay complements [CLAUDE.md](CLAUDE.md) and `README.md` crate map.

## System Map

- `crates/gateway`: HTTP/WS gateway, auth, RPC dispatch, channel + chat wiring.
- `crates/chat` + `crates/agents`: run loop, tool orchestration, streaming events.
- `crates/providers`: provider registry and concrete provider implementations (`openai-codex`, etc.).
- `crates/telegram`: Telegram channel plugin, polling loop, outbound renderer.
- `crates/config`: schema, validation, config loading and template generation.

## Invariants

- Provider auth tokens are persisted through `moltis-oauth` token store.
- Channel plugins remain isolated behind `moltis-channels` traits.
- Gateway protocol compatibility stays aligned with protocol v4 structs.
- Security gates (origin checks, auth middleware, SSRF controls) stay enabled.

## Dependency Boundaries

- Gateway consumes services through traits; business logic remains in crate-specific modules.
- Channel-specific logic does not leak into generic chat/runtime paths.
- OAuth provider behavior is encapsulated in provider + oauth crates.

## Extension Points

- Add new channels via `moltis-channels::ChannelPlugin`.
- Add/modify provider behavior via `crates/providers` registry hooks.
- Add RPC methods via `crates/gateway/src/methods/*` and schema descriptors.
