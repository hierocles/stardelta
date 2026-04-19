import * as fs from "fs";
import { JSON_READER } from "kryo-json/json-reader";
import * as sysPath from "path";
import { CompressionMethod, Movie } from "swf-types";
import { $Movie } from "swf-types/movie";

import { emitSwf } from "../lib/index.mjs";

async function main(): Promise<void> {
  if (process.argv.length < 3) {
    console.error("Missing input path");
    return;
  }
  const filePath: string = process.argv[2];
  const absFilePath: string = sysPath.resolve(filePath);
  const raw: string = fs.readFileSync(absFilePath, {encoding: "utf8"});
  const movie: Movie = $Movie.read(JSON_READER, raw);
  const movieBytes: Uint8Array = emitSwf(movie, CompressionMethod.None);
  fs.writeFileSync("movie.swf", movieBytes, {encoding: null});
}

main()
  .catch((err: Error): never => {
    console.error(err.stack);
    process.exit(1);
  });
