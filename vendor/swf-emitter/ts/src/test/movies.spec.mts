import chai from "chai";
import fs from "fs";
import { JSON_READER } from "kryo-json/json-reader";
import { JSON_VALUE_WRITER } from "kryo-json/json-value-writer";
import sysPath from "path";
import { parseSwf } from "swf-parser";
import { CompressionMethod } from "swf-types";
import { $Movie, Movie } from "swf-types/movie";

import { emitSwf } from "../lib/index.mjs";
import meta from "./meta.mjs";
import { readTextFile } from "./utils.mjs";

const PROJECT_ROOT: string = sysPath.join(meta.dirname, "..");
const MOVIE_SAMPLES_ROOT: string = sysPath.join(PROJECT_ROOT, "..", "tests", "movies");

// `BLACKLIST` can be used to forcefully skip some tests.
const BLACKLIST: ReadonlySet<string> = new Set([]);
// `WHITELIST` can be used to only enable a few tests.
const WHITELIST: ReadonlySet<string> = new Set([
  // "hello-world",
]);

describe("movies", function () {
  this.timeout(300000); // The timeout is this high due to CI being extremely slow

  for (const sample of getSamples()) {
    it(sample.name, async function () {
      const valueJson: string = await readTextFile(sample.astPath);
      const value: Movie = $Movie.read(JSON_READER, valueJson);

      const actualBytes: Uint8Array = emitSwf(value, CompressionMethod.None);

      fs.writeFileSync(sysPath.join(MOVIE_SAMPLES_ROOT, sample.name, "local-main.ts.swf"), actualBytes);

      const actualMovie: Movie = parseSwf(actualBytes);

      try {
        chai.assert.isTrue($Movie.equals(actualMovie, value));
      } catch (err) {
        chai.assert.strictEqual(
          JSON.stringify($Movie.write(JSON_VALUE_WRITER, actualMovie), null, 2),
          valueJson,
        );
        throw err;
      }
    });
  }
});

interface Sample {
  name: string;
}

interface Sample {
  name: string;
  moviePath: string;
  astPath: string;
}

function* getSamples(): IterableIterator<Sample> {
  for (const dirEnt of fs.readdirSync(MOVIE_SAMPLES_ROOT, {withFileTypes: true})) {
    if (!dirEnt.isDirectory()) {
      continue;
    }
    const testName: string = dirEnt.name;
    const testPath: string = sysPath.join(MOVIE_SAMPLES_ROOT, testName);

    if (BLACKLIST.has(testName)) {
      continue;
    } else if (WHITELIST.size > 0 && !WHITELIST.has(testName)) {
      continue;
    }

    const moviePath: string = sysPath.join(testPath, "main.swf");
    const astPath: string = sysPath.join(testPath, "ast.json");

    yield {name: testName, moviePath, astPath};
  }
}
