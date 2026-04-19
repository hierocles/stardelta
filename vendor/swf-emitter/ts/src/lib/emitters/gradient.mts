import { WritableByteStream } from "@open-flash/stream";
import incident from "incident";
import { Uint2, Uint8 } from "semantic-types";
import { ColorSpace, ColorStop, Gradient, GradientSpread, MorphColorStop, MorphGradient } from "swf-types";

import { emitSRgb8, emitStraightSRgba8 } from "./basic-data-types.mjs";

const GRADIENT_SPREAD_TO_CODE: Map<GradientSpread, Uint2> = new Map([
  [GradientSpread.Pad, 0 as Uint2],
  [GradientSpread.Reflect, 1 as Uint2],
  [GradientSpread.Repeat, 2 as Uint2],
]);

const COLOR_SPACE_TO_CODE: Map<ColorSpace, Uint2> = new Map([
  [ColorSpace.LinearRgb, 1 as Uint2],
  [ColorSpace.SRgb, 0 as Uint2],
]);

export function emitGradient(byteStream: WritableByteStream, value: Gradient, withAlpha: boolean): void {
  const spreadCode: Uint2 | undefined = GRADIENT_SPREAD_TO_CODE.get(value.spread);
  const colorSpaceCode: Uint2 | undefined = COLOR_SPACE_TO_CODE.get(value.colorSpace);
  if (spreadCode === undefined) {
    throw new incident.Incident("UnexpectedSpread");
  }
  if (colorSpaceCode === undefined) {
    throw new incident.Incident("UnexpectedColorSpace");
  }

  const flags: Uint8 = 0
    | ((value.colors.length & 0x0f) << 0)
    | ((colorSpaceCode & 0b11) << 4)
    | ((spreadCode & 0b11) << 6);
  byteStream.writeUint8(flags);

  for (const colorStop of value.colors) {
    emitColorStop(byteStream, colorStop, withAlpha);
  }
}

export function emitColorStop(byteStream: WritableByteStream, value: ColorStop, withAlpha: boolean): void {
  byteStream.writeUint8(value.ratio);
  if (withAlpha) {
    emitStraightSRgba8(byteStream, value.color);
  } else {
    emitSRgb8(byteStream, value.color);
  }
}

export function emitMorphGradient(byteStream: WritableByteStream, value: MorphGradient): void {
  const spreadCode: Uint2 | undefined = GRADIENT_SPREAD_TO_CODE.get(value.spread);
  const colorSpaceCode: Uint2 | undefined = COLOR_SPACE_TO_CODE.get(value.colorSpace);
  if (spreadCode === undefined) {
    throw new incident.Incident("UnexpectedSpread");
  }
  if (colorSpaceCode === undefined) {
    throw new incident.Incident("UnexpectedColorSpace");
  }

  const flags: Uint8 = 0
    | ((value.colors.length & 0x0f) << 0)
    | ((colorSpaceCode & 0b11) << 4)
    | ((spreadCode & 0b11) << 6);
  byteStream.writeUint8(flags);

  for (const colorStop of value.colors) {
    emitMorphColorStop(byteStream, colorStop);
  }
}

export function emitMorphColorStop(byteStream: WritableByteStream, value: MorphColorStop): void {
  emitColorStop(byteStream, {ratio: value.ratio, color: value.color}, true);
  emitColorStop(byteStream, {ratio: value.morphRatio, color: value.morphColor}, true);
}
