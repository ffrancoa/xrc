# CLAUDE.md

`xrc` is a Rust CLI that scrapes coding-problem pages into ready-to-solve Rust
cargo projects, then runs their tests to track progress.

## Approach
- Read existing files before writing. Thorough in reasoning, concise in output.
- No sycophantic openers or closing fluff. No emojis or em-dashes.
- Do not add documentation to the code unless asked.

## Conventions
- Ask before committing; never push to the default branch without authorization.
- Commit messages: one title line ending in a period. No body, no co-author trailer.
- No references to any prior/original implementation in code, comments, or docs.
- Output is deterministic, enforced byte-for-byte by golden tests (`tests/golden.rs`)
  and insta snapshots; line endings are LF (`.gitattributes`). Regenerate fixtures on
  purpose and review the diff — never accept it blindly.
- Docs/README tone: warm, human, minimalist.

## Releasing
- Bump `version` in `Cargo.toml` (single source of truth; PyPI reads it via maturin).
- Push a `v*.*.*` tag → `release.yml`: tests → binaries + wheels → crates.io + PyPI
  (trusted publishing) → GitHub Release. Targets: Linux x86_64, macOS aarch64 (ARM
  only), Windows x86_64. A tag publishes even for `-rc`; rehearse via `test.yml` by
  pushing to `main`.

## Commands
```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo run -- pull <url>
cargo run -- check [--recheck] [--verbose]
```

See `.claude/skills/` for deeper procedures (docs, releasing, scrapers).
