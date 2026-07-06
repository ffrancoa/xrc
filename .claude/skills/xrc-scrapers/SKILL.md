---
name: xrc-scrapers
description: Use when adding or modifying a site scraper in src/scrapers/ (the ProblemData contract, registration, tests).
---

# Working on xrc scrapers

Each site is one module under `src/scrapers/` exposing
`extract_problem_parts(url) -> Result<ProblemData>`, with `fetch` and `parse`
split so parsing is testable offline.

## Adding a scraper

1. Produce a `ProblemData` (the contract in `src/scrapers/mod.rs`) — that struct is
   the only interface between a scraper and `renderer.rs`.
2. Register the host in the `SCRAPERS` table in `src/scrapers/mod.rs`; `find_scraper`
   substring-matches the URL's netloc.
3. Keep `fetch` (network) and `parse` (pure, over a `&str`) separate.
4. Add golden fixtures under `tests/fixtures/golden/<name>/` and, if rendering, an
   insta snapshot. Output is enforced **byte-for-byte** — review any diff, never
   accept it blindly.

## Existing scrapers

- **dmoj** (`dmoj.rs`): scrapes HTML with the `scraper` crate. Direct fetch is
  blocked by Cloudflare (403), so it falls back to the Wayback Machine. The fallback
  resolves the **latest** snapshot via the availability API
  (`archive.org/wayback/available?url=...`, called **without** a timestamp — a
  future timestamp returns empty). Do not pin a fixed year.
- **leetcode** (`leetcode.rs`): uses LeetCode's GraphQL API. When `rust_signature`
  is present the generated fn uses the typed signature; otherwise a
  `fn(input: &str) -> String` fallback. Some parsing quirks are deliberate — see the
  comments and tests before "fixing" them.

## Before finishing

Run `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`.
