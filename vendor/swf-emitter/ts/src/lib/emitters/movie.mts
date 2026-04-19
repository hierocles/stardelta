import { WritableByteStream, WritableStream } from "@open-flash/stream";
import incident from "incident";
import { UintSize } from "semantic-types";
import { CompressionMethod, Header, Movie, SwfSignature } from "swf-types";
import zlib from "zlib";

import { emitRect } from "./basic-data-types.mjs";
import { emitTagString } from "./tags.mjs";

const SWF_SIGNATURE_SIZE: UintSize = 8;

export function emitSwf(byteStream: WritableByteStream, value: Movie, compressionMethod: CompressionMethod): void {
  const movieStream: WritableByteStream = new WritableStream();
  emitMovie(movieStream, value);
  const uncompressedFileLength: UintSize = SWF_SIGNATURE_SIZE + movieStream.bytePos;
  const signature: SwfSignature = {
    compressionMethod,
    swfVersion: value.header.swfVersion,
    uncompressedFileLength,
  };
  emitSwfSignature(byteStream, signature);
  switch (compressionMethod) {
    case CompressionMethod.Deflate: {
      const movieBytes: Uint8Array = movieStream.getBytes();
      byteStream.writeBytes(zlib.deflateSync(movieBytes));
      break;
    }
    case CompressionMethod.Lzma: {
      throw new incident.Incident("NotImplemented", "Lzma support");
    }
    case CompressionMethod.None: {
      byteStream.write(movieStream);
      break;
    }
    default: {
      throw new incident.Incident("UnexpectedCompressionMethod");
    }
  }
}

export function emitSwfSignature(byteStream: WritableByteStream, value: SwfSignature): void {
  emitCompressionMethod(byteStream, value.compressionMethod);
  byteStream.writeUint8(value.swfVersion);
  byteStream.writeUint32LE(value.uncompressedFileLength);
}

export function emitCompressionMethod(byteStream: WritableByteStream, value: CompressionMethod): void {
  const COMPRESSION_TO_CHUNK: Map<CompressionMethod, Uint8Array> = new Map([
    [CompressionMethod.Deflate, new Uint8Array([0x43, 0x57, 0x53])],
    [CompressionMethod.Lzma, new Uint8Array([0x5a, 0x57, 0x53])],
    [CompressionMethod.None, new Uint8Array([0x46, 0x57, 0x53])],
  ]);

  const chunk: Uint8Array | undefined = COMPRESSION_TO_CHUNK.get(value);
  if (chunk === undefined) {
    throw new incident.Incident("UnexpectedCompressionMethod");
  }
  byteStream.writeBytes(chunk);
}

function emitMovie(byteStream: WritableByteStream, value: Movie): void {
  emitHeader(byteStream, value.header);
  emitTagString(byteStream, value.tags, value.header.swfVersion);
}

export function emitHeader(byteStream: WritableByteStream, value: Header): void {
  emitRect(byteStream, value.frameSize);
  byteStream.writeUint16LE(value.frameRate.epsilons);
  byteStream.writeUint16LE(value.frameCount);
}
