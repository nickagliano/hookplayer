# Releasing hookplayer

## Prerequisites

- Rust toolchain installed (for local verification before tagging)

## Nice to have (to check releases via CLI)

- [GitHub CLI](https://cli.github.com/) (`gh`) installed and authenticated

## Steps

### 1. Bump the version

Update the version in `Cargo.toml`:

```toml
[package]
version = "x.y.z"
```

### 2. Verify it builds locally

```sh
cargo build --release
```

### 3. Commit and tag

```sh
git add Cargo.toml Cargo.lock
git commit -m "release v x.y.z"
git tag vx.y.z
git push origin master --tags
```

### 4. Wait for CI

The `release.yml` workflow triggers automatically on the tag push. It builds binaries for:

| Asset                      | Platform         |
|----------------------------|------------------|
| `hookplayer-macos-aarch64` | macOS Apple Silicon |
| `hookplayer-macos-x86_64`  | macOS Intel      |
| `hookplayer-linux-x86_64`  | Linux x86_64     |
| `hookplayer-linux-aarch64` | Linux ARM64      |

Monitor progress at: https://github.com/nickagliano/hookplayer/actions

### 5. Verify the release

Once the workflow completes, check that the release looks right:

```sh
gh release view vx.y.z
```

## Versioning

This project follows [Semantic Versioning](https://semver.org/):

- `MAJOR` — breaking changes (config format, CLI interface)
- `MINOR` — new features, backwards compatible
- `PATCH` — bug fixes

## Hotfixes

If a release has a critical bug, fix it on `master`, then tag a patch release (e.g. `v0.1.1`) following the same steps above. Do not delete or re-tag existing releases.
