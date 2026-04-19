<a href="https://github.com/open-flash/open-flash">
    <img src="https://raw.githubusercontent.com/open-flash/open-flash/master/logo.png"
    alt="Open Flash logo" title="Open Flash" align="right" width="64" height="64" />
</a>

# SWF Emitter (Typescript)

[![npm](https://img.shields.io/npm/v/swf-emitter.svg)](https://www.npmjs.com/package/swf-emitter)
[![GitHub repository](https://img.shields.io/badge/Github-open--flash%2Fswf--emitter-blue.svg)](https://github.com/open-flash/swf-emitter)
[![Build status](https://img.shields.io/travis/com/open-flash/swf-emitter/master.svg)](https://travis-ci.com/open-flash/swf-emitter)

SWF emitter implemented in Typescript, for Node and browsers.
Converts [`swf-types` movies][swf-types] to bytes.

## Usage

```typescript
import fs from "fs";
import { CompressionMethod } from "swf-types";
import { Movie } from "swf-types/movie";
import { emitSwf } from "swf-emitter";

function main(): void {
  const movie: Movie = fs.readFileSync("movie.json");
  const bytes: Uint8Array = emitSwf(movie, CompressionMethod.Deflate);
  fs.writeFileSync("movie.swf", bytes);
}

main();
```

## Contributing

This repo uses Git submodules for its test samples:

```sh
# Clone with submodules
git clone --recurse-submodules git://github.com/open-flash/swf-emitter.git
# Update submodules for an already-cloned repo
git submodule update --init --recursive --remote
```

This library uses Gulp and npm for its builds, yarn is recommended for the
dependencies.

```
npm install
# work your changes...
npm test
```

Prefer non-`master` branches when sending a PR so your changes can be rebased if
needed. All the commits must be made on top of `master` (fast-forward merge).
CI must pass for changes to be accepted.

[swf-types]: https://github.com/open-flash/swf-types
