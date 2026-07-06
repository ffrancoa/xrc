---
name: xrc-release
description: Use when cutting an xrc release or changing the release/publish pipeline (crates.io, PyPI, GitHub binaries).
---

# Releasing xrc

Releases are tag-driven. `.github/workflows/release.yml` fires on a `v*.*.*` tag:
`test` → `build-binaries` + `build-wheels`/`build-sdist` → `publish-crate`,
`publish-pypi`, `create-github-release`.

## Cutting a release

1. Bump `version` in `Cargo.toml` — the single source of truth. `pyproject.toml`
   uses `dynamic = ["version"]`, so maturin reads it from there; do not duplicate it.
2. Commit (ask first; one title line ending in a period).
3. Tag and push: `git tag vX.Y.Z && git push origin vX.Y.Z`.

## What ships

- Prebuilt targets (both binaries and wheels): Linux x86_64, **macOS aarch64 only**
  (no x86_64 on macOS), Windows x86_64.
- crates.io and PyPI both via **trusted publishing (OIDC)**, gated on the `release`
  GitHub environment. No tokens stored.

## Gotchas

- A tag publishes the `Cargo.toml` version to **both** registries even for `-rc`
  tags (the version comes from the manifest, not the tag). To rehearse without
  publishing, push to `main` and watch `test.yml` only — it runs no publish jobs.
- crates.io versions are immutable: never re-tag a version already published there.
  If a release fails before `publish-crate` runs, it is safe to move the tag.
- Trusted publisher setup is one-time on each registry's website. PyPI allows a
  "pending publisher" before the first publish; crates.io generally needs the crate
  to exist first (reserve the name with one manual `cargo publish`).
- Line endings must stay LF (`.gitattributes`) or the golden tests fail on Windows.
