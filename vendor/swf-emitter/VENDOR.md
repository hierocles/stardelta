# Vendored `swf-emitter` (Rust crate: `rs/`)

This directory contains a **vendored copy** of [open-flash/swf-emitter](https://github.com/open-flash/swf-emitter), the SWF binary emitter used by StarDelta.

## Source

- **Upstream repository:** https://github.com/open-flash/swf-emitter  
- **Rust crate path:** `rs/` (see [`rs/Cargo.toml`](rs/Cargo.toml))  
- **Version:** 0.14.0 (aligned with `swf-parser` / `swf-types` 0.14.x on crates.io)

The tree was cloned from the default branch of the upstream repo. The previous `git` dependency pointed at a fork with an `implement-import-assets` branch; that remote was not available for automated clone, so StarDelta tracks **open-flash** directly. If import-assets behavior is required again, port those commits onto this vendor tree.

## License

The emitter is licensed under **AGPL-3.0-or-later** (see [`rs/LICENSE.md`](rs/LICENSE.md) and [`rs/README.md`](rs/README.md)). StarDelta remains MIT; combining with AGPL code has licensing implications for distribution—retain upstream license notices when redistributing.

## Maintenance

- Prefer editing files under `vendor/swf-emitter/rs/` for emitter fixes.  
- Do not run `git pull` inside this folder; refresh by replacing the tree from a known upstream commit and updating this file.
