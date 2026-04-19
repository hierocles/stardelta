import { WritableBitStream, WritableByteStream, WritableStream } from "@open-flash/stream";
import incident from "incident";
import { Uint2, Uint5, Uint8, Uint16, UintSize } from "semantic-types";
import { FillStyleType } from "swf-types";
import * as fillStyles from "swf-types/fill-styles/index";
import { JoinStyleType } from "swf-types/join-styles/_type";
import { MorphFillStyle } from "swf-types/morph-fill-style";
import { MorphLineStyle } from "swf-types/morph-line-style";
import { MorphShapeRecord } from "swf-types/morph-shape-record";
import { MorphShapeStyles } from "swf-types/morph-shape-styles";
import { MorphShape } from "swf-types/morph-shape";
import { ShapeRecordType } from "swf-types/shape-records/_type";
import { MorphStyleChange } from "swf-types/shape-records/morph-style-change";

import { getSintMinBitCount, getUintBitCount } from "../get-bit-count.mjs";
import { emitMatrix, emitStraightSRgba8 } from "./basic-data-types.mjs";
import { emitMorphGradient } from "./gradient.mjs";
import { capStyleToCode, emitEdgeBits, emitListLength, joinStyleToCode } from "./shape.mjs";

export enum MorphShapeVersion {
  MorphShape1 = 1,
  MorphShape2 = 2,
}

export function emitMorphShape(
  byteStream: WritableByteStream,
  value: MorphShape,
  morphShapeVersion: MorphShapeVersion,
): void {
  const shapeStream: WritableStream = new WritableStream();
  const startShapeSize: UintSize = emitMorphShapeBits(shapeStream, value, morphShapeVersion);
  byteStream.writeUint32LE(startShapeSize);
  byteStream.write(shapeStream);
}

/**
 * @param bitStream Bitstream where the morph shape should be emitted
 * @param value Morph shape to emit
 * @param morphShapeVersion Morphshape version to use
 * @return Size of the start shape in bytes
 */
export function emitMorphShapeBits(
  bitStream: WritableBitStream,
  value: MorphShape,
  morphShapeVersion: MorphShapeVersion,
): UintSize {
  const [fillBits, lineBits] = emitMorphShapeStylesBits(bitStream, value.initialStyles, morphShapeVersion);
  emitMorphShapeStartRecordStringBits(bitStream, value.records, fillBits, lineBits, morphShapeVersion);
  bitStream.align();

  const result: UintSize = bitStream.bytePos;

  // `0` for style bits: there are no style changes in the end state.
  bitStream.writeUint32Bits(4, 0);
  bitStream.writeUint32Bits(4, 0);
  emitMorphShapeEndRecordStringBits(bitStream, value.records);
  bitStream.align();

  return result;
}

/**
 *
 * @return [fillBits, lineBits]
 */
export function emitMorphShapeStylesBits(
  bitStream: WritableBitStream,
  value: MorphShapeStyles,
  morphShapeVersion: MorphShapeVersion,
): [UintSize, UintSize] {
  const byteStream: WritableByteStream = bitStream.asByteStream();
  emitMorphFillStyleList(byteStream, value.fill);
  emitMorphLineStyleList(byteStream, value.line, morphShapeVersion);
  const fillBits: UintSize = getUintBitCount(value.fill.length);
  const lineBits: UintSize = getUintBitCount(value.line.length);
  bitStream.writeUint32Bits(4, fillBits);
  bitStream.writeUint32Bits(4, lineBits);
  return [fillBits, lineBits];
}

export function emitMorphShapeStartRecordStringBits(
  bitStream: WritableBitStream,
  value: MorphShapeRecord[],
  fillBits: UintSize,
  lineBits: UintSize,
  morphShapeVersion: MorphShapeVersion,
): void {
  for (const record of value) {
    if (record.type === ShapeRecordType.StyleChange) {
      bitStream.writeBoolBits(false); // isEdge
      [fillBits, lineBits] = emitMorphStyleChangeBits(bitStream, record, fillBits, lineBits, morphShapeVersion);
    } else {
      // assert(record.type === ShapeRecordType.Edge)
      bitStream.writeBoolBits(true); // isEdge
      emitEdgeBits(bitStream, record);
    }
  }
  bitStream.writeUint16Bits(6, 0);
}

export function emitMorphShapeEndRecordStringBits(
  bitStream: WritableBitStream,
  value: MorphShapeRecord[],
): void {
  for (const record of value) {
    if (record.type === ShapeRecordType.StyleChange) {
      if (record.moveTo === undefined) {
        continue;
      }
      bitStream.writeBoolBits(false); // isEdge
      const flags: Uint5 = 0b00001; // Pure `moveTo`
      bitStream.writeUint16Bits(5, flags);
      if (record.morphMoveTo === undefined) {
        throw new incident.Incident("UndefinedEndMoveTo");
      }
      const bitCount: UintSize = getSintMinBitCount(record.morphMoveTo.x, record.morphMoveTo.y);
      bitStream.writeUint16Bits(5, bitCount);
      bitStream.writeSint32Bits(bitCount, record.morphMoveTo.x);
      bitStream.writeSint32Bits(bitCount, record.morphMoveTo.y);
    } else {
      // assert(record.type === ShapeRecordType.Edge)
      bitStream.writeBoolBits(true); // isEdge
      emitEdgeBits(
        bitStream,
        {
          type: ShapeRecordType.Edge,
          delta: record.morphDelta,
          controlDelta: record.morphControlDelta,
        },
      );
    }
  }
  bitStream.writeUint16Bits(6, 0);
}

export function emitMorphStyleChangeBits(
  bitStream: WritableBitStream,
  value: MorphStyleChange,
  fillBits: UintSize,
  lineBits: UintSize,
  morphShapeVersion: MorphShapeVersion,
): [UintSize, UintSize] {
  const hasMoveTo: boolean = value.moveTo !== undefined;
  const hasNewLeftFill: boolean = value.leftFill !== undefined;
  const hasNewRightFill: boolean = value.rightFill !== undefined;
  const hasNewLineStyle: boolean = value.lineStyle !== undefined;
  const hasNewStyles: boolean = value.newStyles !== undefined;

  const flags: Uint5 = 0
    | (hasMoveTo ? 1 << 0 : 0)
    | (hasNewLeftFill ? 1 << 1 : 0)
    | (hasNewRightFill ? 1 << 2 : 0)
    | (hasNewLineStyle ? 1 << 3 : 0)
    | (hasNewStyles ? 1 << 4 : 0);

  bitStream.writeUint32Bits(5, flags);

  if (hasMoveTo) {
    const bitCount: UintSize = getSintMinBitCount(value.moveTo!.x, value.moveTo!.y);
    bitStream.writeUint16Bits(5, bitCount);
    bitStream.writeSint32Bits(bitCount, value.moveTo!.x);
    bitStream.writeSint32Bits(bitCount, value.moveTo!.y);
  }

  if (hasNewLeftFill) {
    bitStream.writeUint16Bits(fillBits, value.leftFill!);
  }
  if (hasNewRightFill) {
    bitStream.writeUint16Bits(fillBits, value.rightFill!);
  }
  if (hasNewLineStyle) {
    bitStream.writeUint16Bits(lineBits, value.lineStyle!);
  }

  if (hasNewStyles) {
    [fillBits, lineBits] = emitMorphShapeStylesBits(bitStream, value.newStyles!, morphShapeVersion);
  }

  return [fillBits, lineBits];
}

export function emitMorphFillStyleList(byteStream: WritableByteStream, value: MorphFillStyle[]): void {
  emitListLength(byteStream, value.length, true);
  for (const fillStyle of value) {
    emitMorphFillStyle(byteStream, fillStyle);
  }
}

export function emitMorphFillStyle(byteStream: WritableByteStream, value: MorphFillStyle): void {
  switch (value.type) {
    case FillStyleType.Bitmap: {
      const code: Uint8 = 0
        | (!value.repeating ? 1 << 0 : 0)
        | (!value.smoothed ? 1 << 1 : 0)
        | 0x40;
      byteStream.writeUint8(code);
      emitMorphBitmapFill(byteStream, value);
      break;
    }
    case FillStyleType.FocalGradient: {
      byteStream.writeUint8(0x13);
      emitMorphFocalGradientFill(byteStream, value);
      break;
    }
    case FillStyleType.LinearGradient: {
      byteStream.writeUint8(0x10);
      emitMorphLinearGradientFill(byteStream, value);
      break;
    }
    case FillStyleType.RadialGradient: {
      byteStream.writeUint8(0x12);
      emitMorphRadialGradientFill(byteStream, value);
      break;
    }
    case FillStyleType.Solid: {
      byteStream.writeUint8(0x00);
      emitMorphSolidFill(byteStream, value);
      break;
    }
    default: {
      throw new incident.Incident("UnexpectedMorphFillStyle");
    }
  }
}

export function emitMorphBitmapFill(byteStream: WritableByteStream, value: fillStyles.MorphBitmap): void {
  byteStream.writeUint16LE(value.bitmapId);
  emitMatrix(byteStream, value.matrix);
  emitMatrix(byteStream, value.morphMatrix);
}

export function emitMorphFocalGradientFill(byteStream: WritableByteStream, value: fillStyles.MorphFocalGradient): void {
  emitMatrix(byteStream, value.matrix);
  emitMatrix(byteStream, value.morphMatrix);
  emitMorphGradient(byteStream, value.gradient);
  byteStream.writeSint16LE(value.focalPoint.epsilons);
  byteStream.writeSint16LE(value.morphFocalPoint.epsilons);
}

export function emitMorphLinearGradientFill(
  byteStream: WritableByteStream,
  value: fillStyles.MorphLinearGradient,
): void {
  emitMatrix(byteStream, value.matrix);
  emitMatrix(byteStream, value.morphMatrix);
  emitMorphGradient(byteStream, value.gradient);
}

export function emitMorphRadialGradientFill(
  byteStream: WritableByteStream,
  value: fillStyles.MorphRadialGradient,
): void {
  emitMatrix(byteStream, value.matrix);
  emitMatrix(byteStream, value.morphMatrix);
  emitMorphGradient(byteStream, value.gradient);
}

export function emitMorphSolidFill(byteStream: WritableByteStream, value: fillStyles.MorphSolid): void {
  emitStraightSRgba8(byteStream, value.color);
  emitStraightSRgba8(byteStream, value.morphColor);
}

export function emitMorphLineStyleList(
  byteStream: WritableByteStream,
  value: MorphLineStyle[],
  morphShapeVersion: MorphShapeVersion,
): void {
  emitListLength(byteStream, value.length, true);
  for (const lineStyle of value) {
    if (morphShapeVersion < MorphShapeVersion.MorphShape2) {
      emitMorphLineStyle1(byteStream, lineStyle);
    } else {
      emitMorphLineStyle2(byteStream, lineStyle);
    }
  }
}

export function emitMorphLineStyle1(byteStream: WritableByteStream, value: MorphLineStyle): void {
  if (value.fill.type !== FillStyleType.Solid) {
    throw new incident.Incident("ExpectedSolidMorphFill");
  }
  byteStream.writeUint16LE(value.width);
  byteStream.writeUint16LE(value.morphWidth);
  emitMorphSolidFill(byteStream, value.fill);
}

export function emitMorphLineStyle2(byteStream: WritableByteStream, value: MorphLineStyle): void {
  byteStream.writeUint16LE(value.width);
  byteStream.writeUint16LE(value.morphWidth);

  const hasFill: boolean = value.fill.type !== FillStyleType.Solid;
  const joinStyleCode: Uint2 = joinStyleToCode(value.join.type);
  const startCapStyleCode: Uint2 = capStyleToCode(value.startCap);
  const endCapStyleCode: Uint2 = capStyleToCode(value.endCap);

  const flags: Uint16 = 0
    | (value.pixelHinting ? 1 << 0 : 0)
    | (value.noVScale ? 1 << 1 : 0)
    | (value.noHScale ? 1 << 2 : 0)
    | (hasFill ? 1 << 3 : 0)
    | ((joinStyleCode & 0b11) << 4)
    | ((startCapStyleCode & 0b11) << 6)
    | ((endCapStyleCode & 0b11) << 8)
    | (value.noClose ? 1 << 10 : 0);
  // Skip bits [11, 15]
  byteStream.writeUint16LE(flags);

  if (value.join.type === JoinStyleType.Miter) {
    byteStream.writeSint16LE(value.join.limit.epsilons);
  }

  if (hasFill) {
    emitMorphFillStyle(byteStream, value.fill);
  } else {
    emitMorphSolidFill(byteStream, value.fill as fillStyles.MorphSolid);
  }
}
