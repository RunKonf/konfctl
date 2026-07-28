# AGENTS.md — konf CLI

Rust CLI (`konf`) for the [Konf](https://konf.app) conference platform. Repo/crate is `konfctl`; the installed binary is `konf`. The lib crate is `konfctl` (`use konfctl::...`).

## RTK (Rust Token Killer)
**Always prefix terminal commands with `rtk`** (e.g., `rtk cargo test`, `rtk git status`). This safely filters output to save context tokens. Use even in command chains (`rtk git add . && rtk git commit -m "m"`).

## Build & Test
```sh
rtk mise run agent-check   # token-optimized clippy + fmt-check + test
rtk mise run agent-clippy  # pedantic lints, short message format
rtk mise run fmt           # format with rustfmt
rtk mise run agent-test    # unit + E2E (quiet mode)
```

## Architecture & Conventions
- **Commands**: `src/commands/<domain>/` (keep thin)
- **Types**: `src/types/`
- **Client**: `src/client.rs` (tRPC HTTP client)
- **Display**: `src/display/` (rendering/colors)
- **Template**: `src/template.rs` (`{{{VAR}}}` substitution)
- **Rules**: Clippy pedantic enabled (no warnings). Return `anyhow::Result<T>`, avoid `unwrap()`.
- **Serde**: `#[serde(rename_all = "camelCase")]`, `#[serde(rename = "_id")]`, `#[serde(default)]`. Enum fallbacks: `#[serde(other)] Unknown` + custom Deserialize/Default.
- **Tests**: `#[cfg(test)]` for unit, `tests/e2e.rs` (wiremock) for E2E.

## Commits & Releases
- Commits **must** use Conventional Commits (e.g., `feat(sponsors): add X`, `fix: Y`).
- Release notes include: `feat`, `fix`, `perf`, `refactor`, `docs`.
- Releases/tags are fully automated via GitHub Actions on `main` push. Do **not** tag manually.

## Agent Mode
Always use `--agent` flag globally for token-optimized execution:
- `rtk konf --agent admin proposals list` → Compact JSON, metadata envelope
- `rtk konf --agent admin sponsors list --compact` → Minimal essential fields only
- `rtk konf --agent admin status` → JSON dashboard
- Mutations return structured data `{"ok": true, "id": "..."}`
- Errors return structured format `{"error_code": "...", "error": "...", "hints": [...]}`
