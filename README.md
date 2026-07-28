# konf

[![CI](https://github.com/RunKonf/konfctl/actions/workflows/ci.yml/badge.svg)](https://github.com/RunKonf/konfctl/actions/workflows/ci.yml)
[![Release](https://github.com/RunKonf/konfctl/actions/workflows/release.yml/badge.svg)](https://github.com/RunKonf/konfctl/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

The command-line surface for **[Konf](https://konf.app)** — the conference platform. Review talk proposals, manage the sponsor pipeline, and run your conference from the terminal.

`konf` is a **first-class programmatic surface**, designed for two kinds of operator: conference organizers at a terminal, and LLM agents acting on their behalf. Every command that renders a table also emits machine-readable JSON, interactive prompts can always be bypassed, and errors come back as structured codes with actionable hints. See [Agent usage](#-agent-usage).

> **Naming:** the repository and crate are `konfctl`; the installed command is `konf`; the platform is Konf.

## ✨ Features

- 🔐 **Browser-based login** — authenticate with GitHub or LinkedIn OAuth, no API keys to manage
- 🤖 **Agent-native** — a global `--agent` flag turns the whole CLI into compact JSON with structured errors and safety guards
- 📋 **Interactive proposal review** — fuzzy-search, filter by status/format, sort by rating, and scroll through details with vim-style keybindings
- 💰 **Sponsor pipeline** — track sponsors from prospect to paid, with contacts, tiers, and contract status
- 📧 **Sponsor emails** — send templated emails with an interactive template picker, `$EDITOR` integration, and variable substitution
- 📊 **JSON output everywhere** — pipe to `jq` or feed into scripts with `--json`
- 🔒 **Signed releases** — every binary carries Sigstore build provenance via GitHub Artifact Attestations
- 🖥️ **Cross-platform** — prebuilt binaries for macOS, Linux, and Windows

## 📦 Installation

### Homebrew (macOS and Linux)

> ⏳ **Forthcoming.** The `RunKonf/homebrew-tap` tap is not published yet, so the commands below do **not** work today. Until the tap is live, use a prebuilt binary or build from source.

Once the tap is published, installation will be:

```sh
brew tap RunKonf/tap
brew install konf
```

### Download a prebuilt binary

Grab the latest build for your platform from [GitHub Releases](https://github.com/RunKonf/konfctl/releases/latest):

```sh
# macOS (Apple Silicon)
curl -LO https://github.com/RunKonf/konfctl/releases/latest/download/konf-aarch64-apple-darwin.tar.gz
tar xzf konf-aarch64-apple-darwin.tar.gz
sudo mv konf /usr/local/bin/

# macOS (Intel)
curl -LO https://github.com/RunKonf/konfctl/releases/latest/download/konf-x86_64-apple-darwin.tar.gz
tar xzf konf-x86_64-apple-darwin.tar.gz
sudo mv konf /usr/local/bin/

# Linux (x86_64)
curl -LO https://github.com/RunKonf/konfctl/releases/latest/download/konf-x86_64-unknown-linux-gnu.tar.gz
tar xzf konf-x86_64-unknown-linux-gnu.tar.gz
sudo mv konf /usr/local/bin/

# Linux (arm64)
curl -LO https://github.com/RunKonf/konfctl/releases/latest/download/konf-aarch64-unknown-linux-gnu.tar.gz
tar xzf konf-aarch64-unknown-linux-gnu.tar.gz
sudo mv konf /usr/local/bin/
```

<details>
<summary>🔒 Verify your download</summary>

**Checksum:**

```sh
curl -LO https://github.com/RunKonf/konfctl/releases/latest/download/SHA256SUMS
sha256sum -c SHA256SUMS --ignore-missing
```

**Build provenance** (requires [GitHub CLI](https://cli.github.com/)):

```sh
gh attestation verify konf-aarch64-apple-darwin.tar.gz --repo RunKonf/konfctl
```

Every release is signed with [Sigstore](https://www.sigstore.dev/) via GitHub Artifact Attestations, so you can verify exactly which commit and which workflow produced your binary.

</details>

### Build from source

Requires [Rust 1.85+](https://rustup.rs/).

```sh
git clone https://github.com/RunKonf/konfctl.git
cd konfctl
cargo install --path .   # installs the `konf` binary
```

## 🚀 Quick start

```sh
konf login          # opens browser → pick your conference
konf status         # verify session

konf admin proposals list                 # interactive fuzzy-search
konf admin proposals list --status accepted,confirmed --sort rating
konf admin proposals list --json | jq '.[] | .title'

konf admin proposals review <id>   # interactive review prompts
konf admin proposals review <id> --content 4 --relevance 3 --speaker 5 --comment "Great talk"

konf admin sponsors list
konf admin sponsors list --status negotiating,closedWon
konf admin sponsors get <id>

konf admin sponsors email <id>                    # interactive template picker
konf admin sponsors email <id> --template <slug>  # use specific template
konf admin sponsors email <id> --message "Hello"  # send direct message
konf admin sponsors email <id> --dry-run          # preview without sending

konf logout         # clear credentials
```

## 📖 Usage

### Authentication

Log in via your browser using GitHub or LinkedIn OAuth:

```sh
konf login
```

This opens a browser window, authenticates you, and lets you select which conference to work with. Your session token is stored locally at `~/.config/konf/config.toml`. Override the location with the `KONF_CONFIG` environment variable.

> You must be registered as a conference organizer to use the admin commands.

Check your current session:

```sh
konf status
```

Log out and remove stored credentials:

```sh
konf logout
```

### Proposals

**Interactive mode** (default) — launches a fuzzy-search menu where you can type to filter, use arrow keys to navigate, and press enter to view details:

```sh
konf admin proposals list
```

**Filter and sort** directly from the command line:

```sh
# Only accepted talks, sorted by review rating (highest first)
konf admin proposals list --status accepted --sort rating

# Lightning talks that are still pending review
konf admin proposals list --status submitted --format lightning_10

# Sort alphabetically by speaker name, ascending
konf admin proposals list --sort speaker --asc
```

**Available filters:**

| Flag       | Values                                                                                                                     |
| ---------- | -------------------------------------------------------------------------------------------------------------------------- |
| `--status` | `submitted`, `accepted`, `confirmed`, `waitlisted`, `rejected`, `withdrawn`, `draft`, `deleted`                            |
| `--format` | `lightning_10`, `presentation_20`, `presentation_25`, `presentation_40`, `presentation_45`, `workshop_120`, `workshop_240` |
| `--sort`   | `created`, `title`, `speaker`, `rating`, `reviews`, `status`                                                               |

**View a single proposal** with full details — speakers, topics, description, outline, and review scores:

```sh
konf admin proposals get <proposal-id>
```

In **interactive mode**, selecting a proposal opens a scrollable detail view:

| Key               | Action            |
| ----------------- | ----------------- |
| `↑` / `k`         | Scroll up         |
| `↓` / `j`         | Scroll down       |
| `Ctrl+U` / `PgUp` | Half-page up      |
| `Ctrl+D` / `PgDn` | Half-page down    |
| `←` / `h`         | Previous proposal |
| `→` / `l`         | Next proposal     |
| `r`               | Start a review    |
| `q` / `Esc`       | Back to list      |

**JSON output** for scripting and automation:

```sh
konf admin proposals list --json
konf admin proposals get <proposal-id> --json
```

### Reviews

**Interactive review** — shows the proposal, then prompts for scores (1–5) and a comment:

```sh
konf admin proposals review <proposal-id>
```

**Non-interactive** — provide all scores and comment as flags:

```sh
konf admin proposals review <proposal-id> \
  --content 4 --relevance 3 --speaker 5 \
  --comment "Clear structure, relevant topic, confident speaker"
```

If the proposal has already been reviewed by you, your previous scores and comment are pre-filled as defaults. You can press `Esc` at any prompt to cancel, and a confirmation summary is shown before submitting.

### Sponsors

View the full sponsor pipeline with status, tier, and contract info:

```sh
konf admin sponsors list
```

**Filter by status:**

```sh
konf admin sponsors list --status prospect,negotiating
konf admin sponsors list --status closedWon
```

| Flag       | Values                                                              |
| ---------- | ------------------------------------------------------------------- |
| `--status` | `prospect`, `contacted`, `negotiating`, `closed-won`, `closed-lost` |

**JSON output:**

```sh
konf admin sponsors list --json
```

Dive into a specific sponsor for contacts, billing details, and notes:

```sh
konf admin sponsors get <sponsor-id>
```

### Sponsor Emails

Send templated emails to sponsor contacts directly from the terminal. Templates are managed in the web UI and delivered with full conference branding.

**Interactive mode** (default) — shows a fuzzy-search template picker, pre-sorted by relevance for the sponsor's status and language:

```sh
konf admin sponsors email <sponsor-id>
```

**Select a specific template:**

```sh
konf admin sponsors email <sponsor-id> --template cold-outreach-en
```

**Send a direct message** (skip templates):

```sh
konf admin sponsors email <sponsor-id> --message "Quick follow-up on our call."
```

**Preview without sending:**

```sh
konf admin sponsors email <sponsor-id> --dry-run
konf admin sponsors email <sponsor-id> --dry-run --json
```

**Edit in your editor before sending:**

```sh
konf admin sponsors email <sponsor-id> --edit
```

| Flag           | Description                                           |
| -------------- | ----------------------------------------------------- |
| `--template`   | Template slug (skip interactive picker)                |
| `--subject`    | Override the email subject                             |
| `--message`    | Use this body directly (skip template selection)       |
| `--edit`       | Open `$EDITOR` to edit the message before sending      |
| `--dry-run`    | Preview the email without sending                      |
| `--json`       | Output as JSON                                         |

Template variables like `{{{SPONSOR_NAME}}}`, `{{{CONTACT_NAMES}}}`, and `{{{CONFERENCE_TITLE}}}` are automatically resolved from the sponsor and conference context.

## 🤖 Agent usage

`konf` is built from the ground up to be fully operable by LLM agents (Claude Code, Cursor, Gemini, and friends). Agent support is not a bolt-on — it is why the CLI has the shape it does. Passing the global `--agent` flag enforces machine-readable output and drastically reduces token waste.

### Token-optimized output

When `--agent` is passed, `konf` automatically:

- Formats all output as compact (single-line) JSON rather than pretty-printed text or tables.
- Bypasses interactive prompts, menus, and UI spinners.
- Limits list outputs to a maximum of 50 items and wraps them in a metadata envelope (`{"data": [...], "_meta": {"truncated": true}}`).
- Returns structured mutation confirmations (`{"ok": true, "id": "..."}`).
- Returns categorized error codes and actionable hints instead of raw human-readable errors.

### Safety guards

Destructive or externally-visible actions (deletions, sponsor emails, speaker broadcasts, contract sends) refuse to run autonomously. In agent mode they return an `AGENT_GUARD_BLOCK` error naming the exact command a human must run in their own terminal. Agents should surface that command, not work around it.

### Contract for scripts and agents

- **Exit codes** — `0` on success, `1` on any error.
- **Streams** — data on stdout, errors on stderr. Safe to pipe stdout into `jq`.
- **Structured errors** — with `--agent`, stderr carries `{"error_code": "...", "error": "...", "hints": [...]}`. Branch on `error_code` (`AUTH_REQUIRED`, `NOT_FOUND`, `CONFERENCE_NOT_SET`, `AGENT_GUARD_BLOCK`, `UNKNOWN_ERROR`), not on prose.
- **Non-interactive** — `--agent` globally suppresses prompts; destructive commands also accept `--yes` / `-y`.
- **Config location** — set `KONF_CONFIG` to point at an alternate config file (useful for CI and sandboxes).

### Key commands for agents

```sh
# Globally enable token-optimized output across any command
konf --agent admin proposals list

# Extreme token compression: only essential fields (id, name, status)
konf --agent admin sponsors list --compact
konf --agent admin speakers list --compact
konf --agent admin proposals list --compact

# Agent schema and capability discovery
konf agent-info --json
konf help-json
```

`konf help-json` emits the full command tree — names, descriptions, and arguments — as JSON, so an agent can ingest the entire CLI surface in one call instead of walking `--help` output. `konf agent-info --json` reports the current environment (conference, auth status, config path), the available capability macros, and any conference-specific agent instructions configured by the organizers.

See [AGENT_OPTIMIZATION.md](AGENT_OPTIMIZATION.md) for the design rationale behind each of these mechanisms.

> **Tip:** for maximum efficiency, agents can use the [RTK (Rust Token Killer)](https://github.com/rtk-ai/rtk) wrapper when invoking `konf` if it is available in the environment (`rtk konf ...`).

### MCP server

🚧 **Planned — not yet available.** A Model Context Protocol server is in the works, which will expose the same operations as MCP tools for agents that prefer a protocol over a subprocess. There is no MCP implementation in this repository today; until it ships, `konf --agent` is the supported programmatic interface.

## 🛠️ Development

### Prerequisites

- [Rust 1.85+](https://rustup.rs/)
- [mise](https://mise.jdx.dev/) (optional, for task runner)

### Build and test

```sh
# Using mise (recommended)
mise run check    # clippy + fmt-check + test (parallel)
mise run build    # release build

# Using cargo directly
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check
cargo test
```

### Project structure

```text
src/
  main.rs         — CLI entry point and argument parsing (clap); builds the `konf` binary
  lib.rs          — public module exports (lib crate `konfctl`)
  auth.rs         — browser-based OAuth flow with local callback server
  client.rs       — tRPC HTTP client
  config.rs       — TOML config read/write (~/.config/konf/)
  template.rs     — {{{VAR}}} template variable substitution
  commands/       — command orchestration
    agent_discovery.rs — agent-info and help-json schema emission
    proposals/    — proposal list, detail, review, filters, interactive mode
    sponsors/     — sponsor list, detail, email sending with template picker
  display/        — terminal output formatting (colors, layout, truncation)
  types/          — API response types with typed enums (serde)
  ui/             — reusable TUI components (pager, spinner, terminal helpers)
tests/
  e2e.rs          — end-to-end tests with wiremock
```

## 📜 History

This repository supersedes [`CloudNativeBergen/cnctl`](https://github.com/CloudNativeBergen/cnctl), where this CLI began life as `cnctl`, the organizer CLI for Cloud Native Days Norway. With the platform now generalized and released as Konf, the tool was renamed to `konf` and rehomed here. `cnctl` remains as the historical record and receives no further development.

Migrating from `cnctl`? The command structure and flags are unchanged — only the name is different. Note that config now lives at `~/.config/konf/config.toml` (was `~/.config/cnctl/`) and the config override variable is `KONF_CONFIG` (was `CNCTL_CONFIG`), so run `konf login` once to establish a session.

## 🤝 Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

[MIT](LICENSE) © Hans Kristian Flaatten
