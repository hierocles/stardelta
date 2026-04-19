<a href="https://github.com/open-flash/open-flash">
    <img src="https://raw.githubusercontent.com/open-flash/open-flash/master/logo.png"
    alt="Open Flash logo" title="Open Flash" align="right" width="64" height="64" />
</a>

# SWF Emitter

[![crates.io](https://img.shields.io/crates/v/swf-emitter.svg)](https://crates.io/crates/swf-emitter)
[![GitHub repository](https://img.shields.io/badge/Github-open--flash%2Fswf--emitter-blue.svg)](https://github.com/open-flash/swf-emitter)
[![Build status](https://img.shields.io/travis/com/open-flash/swf-emitter/master.svg)](https://travis-ci.com/open-flash/swf-emitter)

SWF emitter implemented in Rust.
Converts [`swf-types` movies][swf-types] to bytes.

## Usage

```rust
use swf_emitter::emit_swf;
use swf_types::{CompressionMethod, Movie};

fn main() {
  let movie: Movie = ...;
  let swf_bytes = emit_swf(&movie, CompressionMethod::None)
    .expect("Failed to emit movie");
}
```

## Features

SWF compression is provided by the following features, enabled by default:

- `deflate`: enable support for `CompressionMethod::Deflate`, using the [`miniz_oxide`](https://github.com/Frommi/miniz_oxide) crate.
- `lzma`: enable support for `CompressionMethod::Lzma`, using the [`lzma-rs`](https://github.com/gendx/lzma-rs) crate.

Disabling these features will cause `emit_swf` to return an error when passed the corresponding `CompressionMethod`.

## Contributing

This repo uses Git submodules for its test samples:

```sh
# Clone with submodules
git clone --recurse-submodules git://github.com/open-flash/swf-emitter.git
# Update submodules for an already-cloned repo
git submodule update --init --recursive --remote
```

This library is a standard Cargo project. You can test your changes with
`cargo test`.

Prefer non-`master` branches when sending a PR so your changes can be rebased if
needed. All the commits must be made on top of `master` (fast-forward merge).
CI must pass for changes to be accepted.

[swf-types]: https://github.com/open-flash/swf-types
