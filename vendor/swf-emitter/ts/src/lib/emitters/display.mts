import { WritableByteStream, WritableStream } from "@open-flash/stream";
import incident from "incident";
import { Uint8,Uint16, Uint32 } from "semantic-types";
import { BlendMode, ClipAction, ClipEventFlags, Filter, filters, FilterType } from "swf-types";

import { emitStraightSRgba8 } from "./basic-data-types.mjs";

export function emitBlendMode(byteStream: WritableByteStream, value: BlendMode): void {
  const BLEND_MODE_TO_CODE: Map<BlendMode, Uint8> = new Map<BlendMode, Uint8>([
    [BlendMode.Normal, 0],
    [BlendMode.Layer, 2],
    [BlendMode.Multiply, 3],
    [BlendMode.Screen, 4],
    [BlendMode.Lighten, 5],
    [BlendMode.Darken, 6],
    [BlendMode.Difference, 7],
    [BlendMode.Add, 8],
    [BlendMode.Subtract, 9],
    [BlendMode.Invert, 10],
    [BlendMode.Alpha, 11],
    [BlendMode.Erase, 12],
    [BlendMode.Overlay, 13],
    [BlendMode.Hardlight, 14],
  ]);

  const code: Uint8 | undefined = BLEND_MODE_TO_CODE.get(value);
  if (code === undefined) {
    throw new incident.Incident("UnexpectedBlendMode");
  }
  byteStream.writeUint8(code);
}

export function emitClipActionString(
  byteStream: WritableByteStream,
  value: ReadonlyArray<ClipAction>,
  extendedEvents: boolean,
): void {
  byteStream.writeUint16LE(0); // Reserved

  const eventUnion: ClipEventFlags = {
    load: false,
    enterFrame: false,
    unload: false,
    mouseMove: false,
    mouseDown: false,
    mouseUp: false,
    keyDown: false,
    keyUp: false,
    data: false,
    initialize: false,
    press: false,
    release: false,
    releaseOutside: false,
    rollOver: false,
    rollOut: false,
    dragOver: false,
    dragOut: false,
    keyPress: false,
    construct: false,
  };

  for (const clipAction of value) {
    eventUnion.load = eventUnion.load || clipAction.events.load;
    eventUnion.enterFrame = eventUnion.enterFrame || clipAction.events.enterFrame;
    eventUnion.unload = eventUnion.unload || clipAction.events.unload;
    eventUnion.mouseMove = eventUnion.mouseMove || clipAction.events.mouseMove;
    eventUnion.mouseDown = eventUnion.mouseDown || clipAction.events.mouseDown;
    eventUnion.mouseUp = eventUnion.mouseUp || clipAction.events.mouseUp;
    eventUnion.keyDown = eventUnion.keyDown || clipAction.events.keyDown;
    eventUnion.keyUp = eventUnion.keyUp || clipAction.events.keyUp;
    eventUnion.data = eventUnion.data || clipAction.events.data;
    eventUnion.initialize = eventUnion.initialize || clipAction.events.initialize;
    eventUnion.press = eventUnion.press || clipAction.events.press;
    eventUnion.release = eventUnion.release || clipAction.events.release;
    eventUnion.releaseOutside = eventUnion.releaseOutside || clipAction.events.releaseOutside;
    eventUnion.rollOver = eventUnion.rollOver || clipAction.events.rollOver;
    eventUnion.rollOut = eventUnion.rollOut || clipAction.events.rollOut;
    eventUnion.dragOver = eventUnion.dragOver || clipAction.events.dragOver;
    eventUnion.dragOut = eventUnion.dragOut || clipAction.events.dragOut;
    eventUnion.keyPress = eventUnion.keyPress || clipAction.events.keyPress;
    eventUnion.construct = eventUnion.construct || clipAction.events.construct;
  }

  emitClipEventFlags(byteStream, eventUnion, extendedEvents);
  for (const clipAction of value) {
    emitClipAction(byteStream, clipAction, extendedEvents);
  }
  if (extendedEvents) {
    byteStream.writeUint32LE(0);
  } else {
    byteStream.writeUint16LE(0);
  }
}

export function emitClipEventFlags(
  byteStream: WritableByteStream,
  value: ClipEventFlags,
  extendedEvents: boolean,
): void {
  const flags: Uint16 = 0
    | (value.load ? 1 << 0 : 0)
    | (value.enterFrame ? 1 << 1 : 0)
    | (value.unload ? 1 << 2 : 0)
    | (value.mouseMove ? 1 << 3 : 0)
    | (value.mouseDown ? 1 << 4 : 0)
    | (value.mouseUp ? 1 << 5 : 0)
    | (value.keyDown ? 1 << 6 : 0)
    | (value.keyUp ? 1 << 7 : 0)
    | (value.data ? 1 << 8 : 0)
    | (value.initialize ? 1 << 9 : 0)
    | (value.press ? 1 << 10 : 0)
    | (value.release ? 1 << 11 : 0)
    | (value.releaseOutside ? 1 << 12 : 0)
    | (value.rollOver ? 1 << 13 : 0)
    | (value.rollOut ? 1 << 14 : 0)
    | (value.dragOver ? 1 << 15 : 0);

  if (!extendedEvents) {
    byteStream.writeUint16LE(flags);
    return;
  }
  const extendedFlags: Uint32 = flags
    | (value.dragOut ? 1 << 16 : 0)
    | (value.keyPress ? 1 << 17 : 0)
    | (value.construct ? 1 << 18 : 0);
  byteStream.writeUint32LE(extendedFlags);
}

export function emitClipAction(byteStream: WritableByteStream, value: ClipAction, extendedEvents: boolean): void {
  emitClipEventFlags(byteStream, value.events, extendedEvents);
  const actionStream: WritableByteStream = new WritableStream();

  if (value.events.keyPress) {
    if (value.keyCode === undefined) {
      throw new incident.Incident("Expected keyCode to be defined");
    }
    actionStream.writeUint8(value.keyCode);
  }
  actionStream.writeBytes(value.actions);

  byteStream.writeUint32LE(actionStream.bytePos);
  byteStream.write(actionStream);
}

export function emitFilterList(byteStream: WritableByteStream, value: ReadonlyArray<Filter>): void {
  byteStream.writeUint8(value.length);
  for (const filter of value) {
    emitFilter(byteStream, filter);
  }
}

export function emitFilter(byteStream: WritableByteStream, value: Filter): void {
  type FilterEmitter = [(byteStream: WritableByteStream, value: Filter) => void, Uint8];
  const FILTER_TYPE_TO_EMITTER: Map<FilterType, FilterEmitter> = new Map([
    [FilterType.Bevel, [emitBevelFilter, 3] as FilterEmitter],
    [FilterType.Blur, [emitBlurFilter, 1] as FilterEmitter],
    [FilterType.Convolution, [emitConvolutionFilter, 5] as FilterEmitter],
    [FilterType.ColorMatrix, [emitColorMatrixFilter, 6] as FilterEmitter],
    [FilterType.DropShadow, [emitDropShadowFilter, 0] as FilterEmitter],
    [FilterType.Glow, [emitGlowFilter, 2] as FilterEmitter],
    [FilterType.GradientBevel, [emitGradientBevelFilter, 7] as FilterEmitter],
    [FilterType.GradientGlow, [emitGradientGlowFilter, 4] as FilterEmitter],
  ]);

  const filterEmitter: FilterEmitter | undefined = FILTER_TYPE_TO_EMITTER.get(value.filter);
  if (filterEmitter === undefined) {
    throw new incident.Incident("InvalidFilterType");
  }
  byteStream.writeUint8(filterEmitter[1]);
  filterEmitter[0](byteStream, value);
}

export function emitBevelFilter(byteStream: WritableByteStream, value: filters.Bevel): void {
  emitStraightSRgba8(byteStream, value.shadowColor);
  emitStraightSRgba8(byteStream, value.highlightColor);
  byteStream.writeSint32LE(value.blurX.epsilons);
  byteStream.writeSint32LE(value.blurY.epsilons);
  byteStream.writeSint32LE(value.angle.epsilons);
  byteStream.writeSint32LE(value.distance.epsilons);
  byteStream.writeSint16LE(value.strength.epsilons);

  const flags: Uint8 = 0
    | ((value.passes & 0x0f) << 0)
    | (value.onTop ? 1 << 4 : 0)
    | (value.compositeSource ? 1 << 5 : 0)
    | (value.knockout ? 1 << 6 : 0)
    | (value.inner ? 1 << 7 : 0);
  byteStream.writeUint8(flags);
}

export function emitBlurFilter(byteStream: WritableByteStream, value: filters.Blur): void {
  byteStream.writeSint32LE(value.blurX.epsilons);
  byteStream.writeSint32LE(value.blurY.epsilons);
  const flags: Uint8 = 0
    // Skip bits [0, 2]
    | ((value.passes & 0x1f) << 3);
  byteStream.writeUint8(flags);
}

export function emitColorMatrixFilter(byteStream: WritableByteStream, value: filters.ColorMatrix): void {
  for (const coefficient of value.matrix) {
    byteStream.writeFloat32LE(coefficient);
  }
}

export function emitConvolutionFilter(byteStream: WritableByteStream, value: filters.Convolution): void {
  byteStream.writeUint8(value.matrixWidth);
  byteStream.writeUint8(value.matrixHeight);
  byteStream.writeFloat32LE(value.divisor);
  byteStream.writeFloat32LE(value.bias);
  for (const coefficient of value.matrix) {
    byteStream.writeFloat32LE(coefficient);
  }
  emitStraightSRgba8(byteStream, value.defaultColor);
  const flags: Uint8 = 0
    | (value.preserveAlpha ? 1 << 0 : 0)
    | (value.clamp ? 1 << 1 : 0);
  byteStream.writeUint8(flags);
}

export function emitDropShadowFilter(byteStream: WritableByteStream, value: filters.DropShadow): void {
  emitStraightSRgba8(byteStream, value.color);
  byteStream.writeSint32LE(value.blurX.epsilons);
  byteStream.writeSint32LE(value.blurY.epsilons);
  byteStream.writeSint32LE(value.angle.epsilons);
  byteStream.writeSint32LE(value.distance.epsilons);
  byteStream.writeSint16LE(value.strength.epsilons);

  const flags: Uint8 = 0
    | ((value.passes & 0x1f) << 0)
    | (value.compositeSource ? 1 << 5 : 0)
    | (value.knockout ? 1 << 6 : 0)
    | (value.inner ? 1 << 7 : 0);
  byteStream.writeUint8(flags);
}

export function emitGlowFilter(byteStream: WritableByteStream, value: filters.Glow): void {
  emitStraightSRgba8(byteStream, value.color);
  byteStream.writeSint32LE(value.blurX.epsilons);
  byteStream.writeSint32LE(value.blurY.epsilons);
  byteStream.writeSint16LE(value.strength.epsilons);

  const flags: Uint8 = 0
    | ((value.passes & 0x1f) << 0)
    | (value.compositeSource ? 1 << 5 : 0)
    | (value.knockout ? 1 << 6 : 0)
    | (value.inner ? 1 << 7 : 0);
  byteStream.writeUint8(flags);
}

export function emitGradientBevelFilter(byteStream: WritableByteStream, value: filters.GradientBevel): void {
  byteStream.writeUint8(value.gradient.length);
  for (const colorStop of value.gradient) {
    emitStraightSRgba8(byteStream, colorStop.color);
  }
  for (const colorStop of value.gradient) {
    byteStream.writeUint8(colorStop.ratio);
  }
  byteStream.writeSint32LE(value.blurX.epsilons);
  byteStream.writeSint32LE(value.blurY.epsilons);
  byteStream.writeSint32LE(value.angle.epsilons);
  byteStream.writeSint32LE(value.distance.epsilons);
  byteStream.writeSint16LE(value.strength.epsilons);

  const flags: Uint8 = 0
    | ((value.passes & 0x0f) << 0)
    | (value.onTop ? 1 << 4 : 0)
    | (value.compositeSource ? 1 << 5 : 0)
    | (value.knockout ? 1 << 6 : 0)
    | (value.inner ? 1 << 7 : 0);
  byteStream.writeUint8(flags);
}

export function emitGradientGlowFilter(byteStream: WritableByteStream, value: filters.GradientGlow): void {
  byteStream.writeUint8(value.gradient.length);
  for (const colorStop of value.gradient) {
    emitStraightSRgba8(byteStream, colorStop.color);
  }
  for (const colorStop of value.gradient) {
    byteStream.writeUint8(colorStop.ratio);
  }
  byteStream.writeSint32LE(value.blurX.epsilons);
  byteStream.writeSint32LE(value.blurY.epsilons);
  byteStream.writeSint32LE(value.angle.epsilons);
  byteStream.writeSint32LE(value.distance.epsilons);
  byteStream.writeSint16LE(value.strength.epsilons);

  const flags: Uint8 = 0
    | ((value.passes & 0x0f) << 0)
    | (value.onTop ? 1 << 4 : 0)
    | (value.compositeSource ? 1 << 5 : 0)
    | (value.knockout ? 1 << 6 : 0)
    | (value.inner ? 1 << 7 : 0);
  byteStream.writeUint8(flags);
}
