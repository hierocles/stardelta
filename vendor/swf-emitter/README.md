<a href="https://github.com/open-flash/open-flash">
    <img src="https://raw.githubusercontent.com/open-flash/open-flash/master/logo.png"
    alt="Open Flash logo" title="Open Flash" align="right" width="64" height="64" />
</a>

# SWF Emitter

[![npm](https://img.shields.io/npm/v/swf-emitter.svg)](https://www.npmjs.com/package/swf-emitter)
[![crates.io](https://img.shields.io/crates/v/swf-emitter.svg)](https://crates.io/crates/swf-emitter)
[![GitHub repository](https://img.shields.io/badge/Github-open--flash%2Fswf--emitter-blue.svg)](https://github.com/open-flash/swf-emitter)
[![Build status](https://img.shields.io/travis/com/open-flash/swf-emitter/master.svg)](https://travis-ci.com/open-flash/swf-emitter)

SWF emitter implemented in Rust and Typescript (Node and browser).
Converts [`swf-types` movies][swf-types] to bytes.

- [Rust implementation](./rs/README.md)
- [Typescript implementation](./ts/README.md)

This library is part of the [Open Flash][ofl] project.

## Usage

- [Rust](./rs/README.md#usage)
- [Typescript](./ts/README.md#usage)

## Goal

The goal of this emitter is to enable the manipulation and compilation of SWF
files. For example, the emitter is used by Open Flash to generate test samples.

The emitter guarantees that parsing back the result will return the same AST.
If you find an example where this invariant is broken, please fill an issue:
this is a bug. The same AST always produces the same output.

## Status

Most of the tags are implemented but some are still missing.
Help is welcome to complete the emitter.

## Contributing

- [Rust](./rs/README.md#contributing)
- [Typescript](./ts/README.md#contributing)

You can also use the library and report any issues you encounter on the Github
issues page.

[ofl]: https://github.com/open-flash/open-flash
[swf-types]: https://github.com/open-flash/swf-types
