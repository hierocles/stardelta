import { WritableBitStream, WritableByteStream, WritableStream } from "@open-flash/stream";
import incident from "incident";
import { Uint2, Uint3, Uint8, UintSize } from "semantic-types";
import { Glyph, LanguageCode, text } from "swf-types";

import { emitRect, emitSRgb8, emitStraightSRgba8 } from "./basic-data-types.mjs";
import { emitGlyph } from "./shape.mjs";

export function emitLanguageCode(byteStream: WritableByteStream, value: LanguageCode): void {
  const VALUE_TO_CODE: Map<LanguageCode, Uint8> = new Map([
    [LanguageCode.Auto, 0],
    [LanguageCode.Latin, 1],
    [LanguageCode.Japanese, 2],
    [LanguageCode.Korean, 3],
    [LanguageCode.SimplifiedChinese, 4],
    [LanguageCode.TraditionalChinese, 5],
  ]);
  const code: Uint8 | undefined = VALUE_TO_CODE.get(value);
  if (code === undefined) {
    throw new incident.Incident("Unexpected value");
  }
  byteStream.writeUint8(code);
}

export function gridFittingToCode(value: text.GridFitting): Uint3 {
  const VALUE_TO_CODE: Map<text.GridFitting, Uint3> = new Map([
    [text.GridFitting.None, 0 as Uint3],
    [text.GridFitting.Pixel, 1 as Uint3],
    [text.GridFitting.SubPixel, 2 as Uint3],
  ]);
  const code: Uint3 | undefined = VALUE_TO_CODE.get(value);
  if (code === undefined) {
    throw new incident.Incident("Unexpected value");
  }
  return code;
}

export function textRendererToCode(value: text.TextRenderer): Uint2 {
  const VALUE_TO_CODE: Map<text.TextRenderer, Uint2> = new Map([
    [text.TextRenderer.Advanced, 1 as Uint2],
    [text.TextRenderer.Normal, 0 as Uint2],
  ]);
  const code: Uint2 | undefined = VALUE_TO_CODE.get(value);
  if (code === undefined) {
    throw new incident.Incident("Unexpected value");
  }
  return code;
}

export function emitTextRecordString(
  byteStream: WritableByteStream,
  value: ReadonlyArray<text.TextRecord>,
  indexBits: UintSize,
  advanceBits: UintSize,
  withAlpha: boolean,
): void {
  for (const record of value) {
    emitTextRecord(byteStream, record, indexBits, advanceBits, withAlpha);
  }
  byteStream.writeUint8(0);
}

export function emitTextRecord(
  byteStream: WritableByteStream,
  value: text.TextRecord,
  indexBits: UintSize,
  advanceBits: UintSize,
  withAlpha: boolean,
): void {
  const hasOffsetX: boolean = value.offsetX !== 0;
  const hasOffsetY: boolean = value.offsetY !== 0;
  const hasColor: boolean = value.color !== undefined;
  const hasFont: boolean = value.fontId !== undefined && value.fontSize !== undefined;

  const flags: Uint8 = 0
    | (hasOffsetX ? 1 << 0 : 0)
    | (hasOffsetY ? 1 << 1 : 0)
    | (hasColor ? 1 << 2 : 0)
    | (hasFont ? 1 << 3 : 0)
    // Skip bits [4, 6]
    | (1 << 7); // Bit 7 must be set (TextRecordType)
  byteStream.writeUint8(flags);

  if (hasFont) {
    byteStream.writeUint16LE(value.fontId!);
  }
  if (hasColor) {
    if (withAlpha) {
      emitStraightSRgba8(byteStream, value.color!);
    } else {
      emitSRgb8(byteStream, value.color!);
    }
  }
  if (hasOffsetX) {
    byteStream.writeSint16LE(value.offsetX);
  }
  if (hasOffsetY) {
    byteStream.writeSint16LE(value.offsetY);
  }
  if (hasFont) {
    byteStream.writeUint16LE(value.fontSize!);
  }
  byteStream.writeUint8(value.entries.length);
  const bitStream: WritableBitStream = byteStream.asBitStream();
  for (const entry of value.entries) {
    bitStream.writeUint32Bits(indexBits, entry.index);
    bitStream.writeSint32Bits(advanceBits, entry.advance);
  }
  bitStream.align();
}

export function emitCsmTableHintBits(bitStream: WritableBitStream, value: text.CsmTableHint): void {
  const VALUE_TO_CODE: Map<text.CsmTableHint, Uint2> = new Map([
    [text.CsmTableHint.Thin, 0 as Uint2],
    [text.CsmTableHint.Medium, 1 as Uint2],
    [text.CsmTableHint.Thick, 2 as Uint2],
  ]);
  const code: Uint2 | undefined = VALUE_TO_CODE.get(value);
  if (code === undefined) {
    throw new incident.Incident("Unexpected value");
  }
  bitStream.writeUint32Bits(2, code);
}

export function emitFontAlignmentZone(byteStream: WritableByteStream, value: text.FontAlignmentZone): void {
  byteStream.writeUint8(value.data.length);
  for (const zoneData of value.data) {
    emitFontAlignmentZoneData(byteStream, zoneData);
  }
  const flags: Uint8 = 0
    | (value.hasX ? 1 << 0 : 0)
    | (value.hasY ? 1 << 1 : 0);
    // Skip bits [2, 7]
  byteStream.writeUint8(flags);
}

export function emitFontAlignmentZoneData(byteStream: WritableByteStream, value: text.FontAlignmentZoneData): void {
  byteStream.writeFloat16LE(value.origin);
  byteStream.writeFloat16LE(value.size);
}

/**
 * @param byteStream
 * @param value
 * @return useWideOffset
 */
export function emitOffsetGlyphs(
  byteStream: WritableByteStream,
  value: ReadonlyArray<Glyph>,
): boolean {
  const endOffsets: UintSize[] = new Array(value.length);
  const glyphStream: WritableStream = new WritableStream();
  for (let i: UintSize = 0; i < value.length; i++) {
    emitGlyph(glyphStream, value[i]);
    endOffsets[i] = glyphStream.bytePos;
  }

  const offsetTableLength: UintSize = endOffsets.length + 1;
  const shortOffsetTableSize: UintSize = offsetTableLength * 2; // 2 bytes per offset
  const maxOffsetWithShortTable: UintSize = shortOffsetTableSize + glyphStream.bytePos;

  const useWideOffsets: boolean = maxOffsetWithShortTable > 0xffff;

  if (useWideOffsets) {
    const wideOffsetTableSize: UintSize = offsetTableLength * 4; // 4 bytes per offset

    byteStream.writeUint32LE(wideOffsetTableSize);
    for (const endOffset of endOffsets) {
      byteStream.writeUint32LE(wideOffsetTableSize + endOffset);
    }
  } else {
    byteStream.writeUint16LE(shortOffsetTableSize);
    for (const endOffset of endOffsets) {
      byteStream.writeUint16LE(shortOffsetTableSize + endOffset);
    }
  }

  byteStream.write(glyphStream);

  return useWideOffsets;
}

export function emitFontLayout(byteStream: WritableByteStream, value: text.FontLayout): void {
  byteStream.writeUint16LE(value.ascent);
  byteStream.writeUint16LE(value.descent);
  byteStream.writeUint16LE(value.leading);
  for (const advance of value.advances) {
    byteStream.writeUint16LE(advance);
  }
  for (const bound of value.bounds) {
    emitRect(byteStream, bound);
  }
  byteStream.writeUint16LE(value.kerning.length);
  for (const kerningRecord of value.kerning) {
    emitKerningRecord(byteStream, kerningRecord);
  }
}

export function emitKerningRecord(byteStream: WritableByteStream, value: text.KerningRecord): void {
  byteStream.writeUint16LE(value.left);
  byteStream.writeUint16LE(value.right);
  byteStream.writeSint16LE(value.adjustment);
}

export function emitTextAlignment(byteStream: WritableByteStream, value: text.TextAlignment): void {
  const TEXT_ALIGNMENT_TO_CODE: Map<text.TextAlignment, Uint8> = new Map([
    [text.TextAlignment.Center, 2],
    [text.TextAlignment.Justify, 3],
    [text.TextAlignment.Left, 0],
    [text.TextAlignment.Right, 1],
  ]);
  const code: Uint8 | undefined = TEXT_ALIGNMENT_TO_CODE.get(value);
  if (code === undefined) {
    throw new incident.Incident("UnexpectedTextAlignment");
  }
  byteStream.writeUint8(code);
}
