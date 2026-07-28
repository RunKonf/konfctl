# Contributing

Thanks for your interest in contributing to the Konf CLI (`konf`)!

## Getting started

1. Fork and clone the repository
2. Install [Rust 1.85+](https://rustup.rs/)
3. Run the checks: `cargo clippy --all-targets -- -D warnings && cargo fmt -- --check && cargo test`

## Development workflow

1. Create a branch from `main`
2. Make your changes
3. Ensure all checks pass: `mise run check` (or run clippy, fmt, and test individually)
4. Submit a pull request

## Code style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Pedantic clippy lints are enforced
- Format with `cargo fmt` (edition 2024)
- Keep commands thin — business logic in domain modules, display logic in `display/`

## Testing

- Unit tests live alongside source code (`#[cfg(test)]` modules)
- Integration tests live in `tests/e2e.rs` using [wiremock](https://crates.io/crates/wiremock)
- All tests must pass before merging

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/) format.
Release notes are auto-generated from these by git-cliff.

```
feat(sponsors): add email template picker
fix: handle missing conference ID
docs: update README with new flags
```

See `cliff.toml` for which prefixes are included/excluded.

## Releases

Releases are fully automated. Every push to `main` that passes CI triggers a
release with cross-compiled binaries and auto-generated release notes. Tags use
date-based versioning (`YYYY.MM.DD-<shortsha>`) — do not create tags manually.
