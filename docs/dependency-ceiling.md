# SWF dependency ceiling

StarDelta converts SWF files to JSON using **swf-parser** 0.14 and back to binary using a **vendored [swf-emitter](https://github.com/open-flash/swf-emitter)** (see [`vendor/swf-emitter/`](../vendor/swf-emitter/)) with **swf-types** 0.14. The [Open Flash](https://github.com/open-flash) projects are no longer actively maintained upstream; behavior is defined by the versions pinned in [`src-tauri/Cargo.toml`](../src-tauri/Cargo.toml) and any local edits under `vendor/swf-emitter/rs/`.

## What “SWF 19 support” means here

- **Binary level:** Any tag or structure that **round-trips** through `parse_swf` → `Movie` → `emit_swf` on your files is supported for full JSON editing (export the `Movie`, edit, re-import).
- **Patch file level:** The modification JSON (`modifications` array) uses `tag` names like `DefineShapeTag` and merges `properties` into the corresponding `swf-types` structs (see [`src-tauri/src/swf_tag_merge.rs`](../src-tauri/src/swf_tag_merge.rs)). Optional fields: `tag_index`, `depth`, `character_id`, `frame_label_name` to target the correct tag when multiple instances exist.
- **Gaps:** If the parser emits `Raw` / `RawBody` for unknown tags, those bytes are preserved when the emitter handles them; otherwise limitations are those of the parser/emitter, not a separate StarDelta cutoff.

## Regression testing

Run `cargo test` in `src-tauri/`; integration tests under `tests/` include a minimal SWF round-trip check.
