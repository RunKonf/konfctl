# Agent Optimization Techniques

`konf` implements several patterns to optimize execution for LLM agents. These optimizations focus on minimizing token usage and providing deterministic, machine-readable interfaces.

## 1. Global Agent State (`--agent`)
A global `--agent` flag enables a machine-readable mode across the entire CLI (`is_agent()`).

**Mechanism:**
- Bypasses interactive elements (spinners, TUI menus, standard-in prompts) which agents cannot handle.
- Enforces strict limits and specific output formats without requiring agents to apply multiple subcommand-specific flags.

## 2. Output Capping & Metadata Envelopes
List commands can easily exhaust an agent's context window with large datasets.

**Mechanism:**
- Output is strictly capped (defaulting to 50 items).
- The returned array is wrapped in a JSON metadata envelope:
  ```json
  {
    "_meta": {
      "total": 312,
      "returned": 50,
      "truncated": true,
      "hint": "Use --limit or other filters to narrow results"
    },
    "data": [ ... ]
  }
  ```
- This prevents the agent from assuming the subset is the complete dataset and provides instructions on how to retrieve remaining records.

## 3. Compact JSON Serialization (`--compact`)
Standard JSON carries significant token overhead due to formatting and comprehensive data models.

**Mechanism:**
- Works in tandem with `--agent` to strip data models to essential fields (`id`, `name`, `status`).
- Serializes output without pretty-printing (no whitespace or newlines).
- Reduces token consumption by 60-90% per request.

## 4. Structured Error Handling
Human-readable errors (prose, stack traces) are difficult to parse programmatically.

**Mechanism:**
- Application errors are caught and formatted as structured JSON:
  ```json
  {
    "error_code": "AUTH_REQUIRED",
    "error": "Authentication required. Valid session not found.",
    "hints": ["Run 'konf login' to authenticate."]
  }
  ```
- Provides a deterministic `error_code` for conditional branching and actionable `hints` to prevent hallucinated recovery attempts.

## 5. Safety Guards (`AGENT_GUARD_BLOCK`)
Destructive actions (e.g., deletions, mass emails) pose risks when executed autonomously.

**Mechanism:**
- The CLI intercepts sensitive commands in agent mode and returns an `AGENT_GUARD_BLOCK` error.
- Execution is halted, forcing the agent to request manual confirmation from the human user.

## 6. Schema Minimization
Agents query help output (`help-json`) to discover CLI capabilities.

**Mechanism:**
- Boilerplate arguments injected by the argument parser (e.g., `--help`, `--version`) are filtered out before JSON serialization.
- Removing repetitive definitions across numerous subcommands saves thousands of tokens during schema ingestion.

## 7. Build & Test Tooling (`mise` tasks)
Standard build tooling often prints hundreds of lines of compiling statuses or repetitive "ok" messages.

**Mechanism:**
- Dedicated agent tasks in `.mise.toml` (`agent-test`, `agent-clippy`) wrap the native Cargo commands with quiet flags (`cargo test -q`) and short message formats (`--message-format=short`).
- This ensures test suites and lints return only a few lines of context unless there is an actual failure, saving massive amounts of context during continuous agent iteration.

## Implementation Details
The core token-efficiency logic is abstracted into `crate::display::print_json_list`. This generic helper manages limits, metadata wrapping, and compact/pretty serialization, applying the agent optimizations consistently across commands.
