import { WritableByteStream, WritableStream } from "@open-flash/stream";
import { Uint8 } from "semantic-types";
import * as swf from "swf-types";
import { CompressionMethod } from "swf-types/compression-method";
import { Movie } from "swf-types/movie";
import { Tag } from "swf-types/tag";

import { emitSwf as emitSwfStream } from "./emitters/movie.mjs";
import { emitTag as emitTagStream } from "./emitters/tags.mjs";

export { swf };

/**
 * Emits an SWF movie as a byte array.
 *
 * @param value SWF movie to emit.
 * @param compressionMethod Compression method for the SWF payload.
 * @returns Corresponding byte array.
 */
export function emitSwf(
  value: Movie,
  compressionMethod: CompressionMethod = CompressionMethod.None,
): Uint8Array {
  const s: WritableByteStream = new WritableStream();
  emitSwfStream(s, value, compressionMethod);
  return s.getBytes();
}

/**
 * Emits an SWF tag as a byte array.
 *
 * @param value SWF tag to emit.
 * @param swfVersion SWF version to use for tags sensible to the version.
 * @returns Corresponding byte array.
 */
export function emitTag(
  value: Tag,
  swfVersion: Uint8,
): Uint8Array {
  const s: WritableByteStream = new WritableStream();
  emitTagStream(s, value, swfVersion);
  return s.getBytes();
}
