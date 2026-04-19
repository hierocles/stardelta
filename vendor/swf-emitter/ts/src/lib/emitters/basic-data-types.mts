import { WritableBitStream, WritableByteStream } from "@open-flash/stream";
import { Uint16, UintSize } from "semantic-types";
import { ColorTransform, ColorTransformWithAlpha, Matrix, Rect, SRgb8, StraightSRgba8 } from "swf-types";

import { getSintMinBitCount } from "../get-bit-count.mjs";

export function emitRect(byteStream: WritableByteStream, value: Rect): void {
  const bitStream: WritableBitStream = byteStream.asBitStream();
  emitRectBits(bitStream, value);
  bitStream.align();
}

export function emitRectBits(bitStream: WritableBitStream, value: Rect): void {
  const nBits: UintSize = getSintMinBitCount(value.xMin, value.xMax, value.yMin, value.yMax);
  bitStream.writeUint16Bits(5, nBits);
  bitStream.writeSint16Bits(nBits, value.xMin);
  bitStream.writeSint16Bits(nBits, value.xMax);
  bitStream.writeSint16Bits(nBits, value.yMin);
  bitStream.writeSint16Bits(nBits, value.yMax);
}

export function emitSRgb8(byteStream: WritableByteStream, value: SRgb8): void {
  byteStream.writeUint8(value.r);
  byteStream.writeUint8(value.g);
  byteStream.writeUint8(value.b);
}

export function emitStraightSRgba8(byteStream: WritableByteStream, value: StraightSRgba8): void {
  byteStream.writeUint8(value.r);
  byteStream.writeUint8(value.g);
  byteStream.writeUint8(value.b);
  byteStream.writeUint8(value.a);
}

export function emitMatrix(byteStream: WritableByteStream, value: Matrix): void {
  const bitStream: WritableBitStream = byteStream.asBitStream();
  emitMatrixBits(bitStream, value);
  bitStream.align();
}

export function emitMatrixBits(bitStream: WritableBitStream, value: Matrix): void {
  if (value.scaleX.valueOf() === 1 && value.scaleY.valueOf() === 1) {
    bitStream.writeBoolBits(false);
  } else {
    bitStream.writeBoolBits(true);
    const nBits: UintSize = getSintMinBitCount(value.scaleX.epsilons, value.scaleY.epsilons);
    bitStream.writeUint16Bits(5, nBits);
    bitStream.writeSint32Bits(nBits, value.scaleX.epsilons);
    bitStream.writeSint32Bits(nBits, value.scaleY.epsilons);
  }
  if (value.rotateSkew0.valueOf() === 0 && value.rotateSkew1.valueOf() === 0) {
    bitStream.writeBoolBits(false);
  } else {
    bitStream.writeBoolBits(true);
    const nBits: UintSize = getSintMinBitCount(value.rotateSkew0.epsilons, value.rotateSkew1.epsilons);
    bitStream.writeUint16Bits(5, nBits);
    bitStream.writeSint32Bits(nBits, value.rotateSkew0.epsilons);
    bitStream.writeSint32Bits(nBits, value.rotateSkew1.epsilons);
  }
  {
    const nBits: UintSize = getSintMinBitCount(value.translateX, value.translateY);
    bitStream.writeUint16Bits(5, nBits);
    bitStream.writeSint16Bits(nBits, value.translateX);
    bitStream.writeSint16Bits(nBits, value.translateY);
  }
}

export function emitColorTransform(byteStream: WritableByteStream, value: ColorTransform): void {
  const bitStream: WritableBitStream = byteStream.asBitStream();
  emitColorTransformBits(bitStream, value);
  bitStream.align();
}

export function emitColorTransformBits(bitStream: WritableBitStream, value: ColorTransform): void {
  const hasAdd: boolean = value.redAdd !== 0 || value.greenAdd !== 0 || value.blueAdd !== 0;
  const hasMult: boolean = value.redMult.valueOf() !== 1
    || value.greenMult.valueOf() !== 1
    || value.blueMult.valueOf() !== 1;

  const toWrite: Uint16[] = [];
  if (hasMult) {
    toWrite.push(value.redMult.epsilons);
    toWrite.push(value.greenMult.epsilons);
    toWrite.push(value.blueMult.epsilons);
  }
  if (hasAdd) {
    toWrite.push(value.redAdd);
    toWrite.push(value.greenAdd);
    toWrite.push(value.blueAdd);
  }

  const bits: UintSize = getSintMinBitCount(...toWrite);

  bitStream.writeBoolBits(hasAdd);
  bitStream.writeBoolBits(hasMult);
  bitStream.writeUint16Bits(4, bits);
  for (const value of toWrite) {
    bitStream.writeSint16Bits(bits, value);
  }
}

export function emitColorTransformWithAlpha(byteStream: WritableByteStream, value: ColorTransformWithAlpha): void {
  const bitStream: WritableBitStream = byteStream.asBitStream();
  emitColorTransformWithAlphaBits(bitStream, value);
  bitStream.align();
}

export function emitColorTransformWithAlphaBits(bitStream: WritableBitStream, value: ColorTransformWithAlpha): void {
  const hasAdd: boolean = value.redAdd !== 0 || value.greenAdd !== 0 || value.blueAdd !== 0 || value.alphaAdd !== 0;
  const hasMult: boolean = value.redMult.valueOf() !== 1
    || value.greenMult.valueOf() !== 1
    || value.blueMult.valueOf() !== 1
    || value.alphaMult.valueOf() !== 1;

  const toWrite: Uint16[] = [];
  if (hasMult) {
    toWrite.push(value.redMult.epsilons);
    toWrite.push(value.greenMult.epsilons);
    toWrite.push(value.blueMult.epsilons);
    toWrite.push(value.alphaMult.epsilons);
  }
  if (hasAdd) {
    toWrite.push(value.redAdd);
    toWrite.push(value.greenAdd);
    toWrite.push(value.blueAdd);
    toWrite.push(value.alphaAdd);
  }

  const bits: UintSize = getSintMinBitCount(...toWrite);

  bitStream.writeBoolBits(hasAdd);
  bitStream.writeBoolBits(hasMult);
  bitStream.writeUint16Bits(4, bits);
  for (const value of toWrite) {
    bitStream.writeSint16Bits(bits, value);
  }
}
