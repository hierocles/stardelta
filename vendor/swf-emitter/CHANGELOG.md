# 0.14.0 (2022-05-08)

- **[Breaking change]** Update to `swf-types@0.14`.

## Rust

- **[Breaking Change]** `emit_swf` now returns `Result<(), SwfEmitError>` (instead of `Result<(), std::io::Error>`).
- **[Feature]** Add enabled-by-default features `deflate` and `lzma`, they add support for compression methods.

# Typescript

- **[Breaking change]** Compile to `.mjs`.
- **[Fix]** Update dependencies.
- **[Internal]** Use Yarn's Plug'n'Play linker.

# 0.13.0 (2021-07-24)

## Rust

- **[Breaking change]** Update to `swf-types@0.13`.
- **[Fix]** Update dependencies.
- **[Internal]** Add clippy support.

## Typescript

- **[Breaking change]** Update to `swf-types@0.13`.
- **[Breaking change]** Drop `lib` prefix and `.js` extension from deep-imports.
- **[Fix]** Update dependencies.

# 0.12.0 (2020-09-05)

- **[Breaking change]** Update to `swf-types@0.12.0`.

## Typescript

- **[Breaking change]** Update to native ESM.
- **[Internal]** Switch from `tslint` to `eslint`.

# 0.11.1 (2020-02-06)

- **[Feature]** Implement emitter for `SymbolClass`.

# 0.11.0 (2020-02-05)

- **[Breaking change]** Update to `swf-types@0.11.0`.
- **[Feature]** Implement emitter for `DoAbc`.

# 0.10.0 (2020-01-20)

- **[Breaking change]** Update to `swf-types@0.10.0`.
- **[Breaking change]** Refactor consumer API. The main export now consists in two simple functions `emitSwf` and `emitTag`, both return a byte array.
- **[Fix]** Add support for `DefineButton1`.
- **[Fix]** Add support for `DefineButtonSound`.
- **[Fix]** Add support for `StartSound`.

### Rust

- **[Fix]** Remove unused dependencies.

# 0.8.0 (2019-07-08)

- **[Breaking change]** Update to `swf-tree@0.8`.
- **[Feature]** Implement emitters for `DefineGlyphFont`, `DefineFontInfo` and `Protect`.

# 0.7.0 (2019-05-21)

- **[Breaking change]** Update to `swf-tree@0.7`.

### Typescript

- **[Internal]** Update build tools.

# 0.3.1 (2019-04-26)

- **[Fix]** Zero-out style bits in morph shape end state.
- **[Fix]** Force long-form tag header for the following concrete tags: `DefineBits`, `DefineBitsJPEG2`, `DefineBitsJPEG3`, `DefineBitsLossless`, `DefineBitsLossless2`, `DefineBitsJPEG4` and `SoundStreamBlock`.

### Rust

- **[Feature]** Implement emitters for the following tags: `DefineBitmap`, `DefineButton`, `DefineDynamicText`, `DefineJpegTables`, `DefineSound`, `ExportAssets`, `FrameLabel`.
- **[Fix]** Fix `LineStyle2` and `MorphLineStyle2` flags.
- **[Fix]** Fix `MorphLineStyle2` with solid fill.
- **[Internal]** Use exhaustive match in `emit_tag`.

### Typescript

- **[Fix]** Use shorter bit count for `ColorTransform` and `ColorTransformWithAlpha`.
- **[Fix]** Fix `ButtonCond` flags.
- **[Fix]** Fix `DefineDynamicText` layout support.

# 0.3.0 (2019-04-24)

### Rust

- **[Feature]** Add Rust implementation.
- **[Feature]** Implement emitters for the following tags: `CsmTextSettings`, `DefineFont`, `DefineFontAlignZones`, `DefineFontName`, `DefineMorphShape`, `DefineSceneAndFrameLabelData`, `DefineShape`, `DefineSprite`, `DefineText`, `DoAction`, `FileAttributes`, `Metadata`, `PlaceObject`, `RemoveObject`, `SetBackgroundColor`, `ShowFrame`.
- **[Feature]** Implement `emit_movie`.

### Typescript

- **[Breaking change]** Update to `swf-tree@0.6.0`.
- **[Fix]** Fix `emitFontAlignmentZone` flags.
- **[Fix]** Fix `emitDefineMorphShapeAny` flags.
- **[Fix]** Use shorter bit counts for shape styles and glyphs.
- **[Fix]** Fix `emitTextRecord` flags.

# 0.2.0 (2019-01-22)

### Typescript

- **[Breaking change]** Update to `swf-tree@0.2.0`.

# 0.1.1 (2019-01-21)

### Typescript

- **[Fix]** Fix `emitPlaceObject`.
- **[Fix]** Fix `emitDefineSprite`.
- **[Fix]** Move stream implementation to `@open-flash/stream`.
- **[Internal]** Add end-to-end tests.

# 0.1.0 (2019-01-12)

### Typescript

- **[Fix]** Update dependencies.

# 0.0.8 (2018-11-07)

### Typescript

- **[Breaking change]** Use opaque bytes for AVM1. Use [open-flash/avm1-emitter](https://github.com/open-flash/avm1-emitter) to convert AVM1 actions to bytes.
