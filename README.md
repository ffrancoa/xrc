# xrc

[![Crates.io](https://img.shields.io/crates/v/xrc.svg)](https://crates.io/crates/xrc)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Turn a coding-problem URL into a ready-to-solve Rust project, then track your
progress as you go.

`xrc` scrapes a problem page, writes a clean cargo project with the statement,
constraints, and sample tests already wired up, and gets out of your way so you
can focus on the fun part: solving it.

## Install

```bash
cargo install xrc
```

## Pull a problem

Point `xrc` at a problem you want to work on:

```bash
xrc pull https://dmoj.ca/problem/aplusb
xrc pull https://leetcode.com/problems/two-sum/
```

This creates a self-contained project under `exercises/`:

```
exercises/
└── a_plus_b/
    ├── Cargo.toml
    ├── src/
    └── tests/
```

The problem statement lives right in the source as comments, and the sample
cases are already turned into tests. Open it up and start writing your solution.

Supported sites today: **dmoj.ca** and **leetcode.com**.

## Check your progress

When you think you have it, let `xrc` run the tests for every exercise and keep
score:

```bash
xrc check
```

Solved problems are remembered, so `check` skips them next time and only runs
what's left. A few flags for when you need them:

```bash
xrc check --verbose   # show the full build and test output
xrc check --recheck   # re-run everything, even already-solved problems
```

Your progress is saved in `exercises/.xrc_progress.json`, so you can pick up
right where you left off.

## License

Released under the [MIT License](LICENSE).
