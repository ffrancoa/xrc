---
name: xrc-docs
description: Use when writing or editing xrc's README or other user-facing docs — tone, structure, and what must stay accurate.
---

# Writing xrc docs

**Tone:** warm, human, minimalist. Short sentences, no fluff, no emojis or em-dashes.

**Tagline:** always the crate description, kept in sync with `Cargo.toml` and
`pyproject.toml`: _"A minimal CLI to give your Rust some exercise."_

**README structure (keep this order):**
1. Title + badges: crates.io, PyPI, license.
2. Tagline, then a short paragraph on what `xrc` does.
3. `## Install` — `cargo install xrc`, plus `pip install xrc` / `uv tool install xrc`.
4. `## Pull a problem` — real examples for dmoj.ca and leetcode.com, show the
   generated `exercises/<name>/` tree.
5. `## Check your progress` — `xrc check` and the `--verbose` / `--recheck` flags.
6. `## License` — MIT.

**Accuracy rules:**
- The CLI surface is the source of truth. Verify commands/flags against
  `cargo run -- --help` (and subcommand help) before documenting them.
- Supported sites today: dmoj.ca and leetcode.com. Do not imply others.
- Install lines are only valid once the crate/package is published; do not add
  install instructions for a registry it is not on yet.
