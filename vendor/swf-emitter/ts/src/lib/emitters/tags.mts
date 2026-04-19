import { WritableBitStream, WritableByteStream, WritableStream } from "@open-flash/stream";
import incident from "incident";
import { Uint2, Uint3, Uint4, Uint8, Uint16, Uint32, UintSize } from "semantic-types";
import { LanguageCode, Tag, tags, TagType } from "swf-types";
import { SoundType } from "swf-types/sound/sound-type";
import { TextAlignment } from "swf-types/text/text-alignment";

import { getSintBitCount, getUintBitCount } from "../get-bit-count.mjs";
import {
  emitColorTransform,
  emitColorTransformWithAlpha,
  emitMatrix,
  emitRect,
  emitSRgb8,
  emitStraightSRgba8,
} from "./basic-data-types.mjs";
import {
  ButtonVersion,
  emitButton2CondActionString,
  emitButtonRecordString, emitButtonSound,
  getMinButtonVersion,
} from "./button.mjs";
import { emitBlendMode, emitClipActionString, emitFilterList } from "./display.mjs";
import { emitMorphShape, MorphShapeVersion } from "./morph-shape.mjs";
import { emitGlyph, emitShape, getMinShapeVersion, ShapeVersion } from "./shape.mjs";
import { audioCodingFormatToCode, emitSoundInfo, soundRateToCode } from "./sound.mjs";
import {
  emitCsmTableHintBits,
  emitFontAlignmentZone,
  emitFontLayout,
  emitLanguageCode,
  emitOffsetGlyphs,
  emitTextAlignment,
  emitTextRecordString,
  gridFittingToCode,
  textRendererToCode,
} from "./text.mjs";

/**
 * Read tags until the end of the stream or "end-of-tags".
 */
export function emitTagString(byteStream: WritableByteStream, value: ReadonlyArray<Tag>, swfVersion: Uint8): void {
  for (const tag of value) {
    emitTag(byteStream, tag, swfVersion);
  }
  emitEndOfTags(byteStream);
}

interface TagHeader {
  code: Uint16;
  length: Uint32;
}

/**
 * Set of tag codes that require a long header.
 */
const LONG_HEADER_REQUIRED: ReadonlySet<Uint16> = new Set([
  6, // DefineBits
  21, // DefineBitsJPEG2
  35, // DefineBitsJPEG3
  20, // DefineBitsLossless
  36, // DefineBitsLossless2
  90, // DefineBitsJPEG4
  19, // SoundStreamBlock
]);

function emitTagHeader(byteStream: WritableByteStream, value: TagHeader): void {
  const SHORT_TAG_MAX_LENGTH: Uint8 = (1 << 6) - 1;
  const isLongRequired: boolean = LONG_HEADER_REQUIRED.has(value.code);
  const isLeadingByteNonZero: boolean = value.length > 0 || (value.code & 0b11) !== 0;

  if (!isLongRequired && value.length < SHORT_TAG_MAX_LENGTH && isLeadingByteNonZero) {
    const codeAndLength: Uint16 = (value.code << 6) | value.length;
    byteStream.writeUint16LE(codeAndLength);
  } else {
    const codeAndLength: Uint16 = (value.code << 6) | SHORT_TAG_MAX_LENGTH;
    byteStream.writeUint16LE(codeAndLength);
    byteStream.writeUint32LE(value.length);
  }
}

function emitEndOfTags(byteStream: WritableByteStream): void {
  byteStream.writeUint16LE(0);
}

// tslint:disable-next-line:cyclomatic-complexity
export function emitTag(byteStream: WritableByteStream, value: Tag, swfVersion: Uint8): void {
  type TagEmitter = number | [(
    byteStream: WritableByteStream,
    value: Tag,
    swfVersion?: Uint8,
  ) => any, number | Map<any, number>];

  const TAG_TYPE_TO_EMITTER: Map<TagType, TagEmitter> = new Map<TagType, TagEmitter>([
    [TagType.CsmTextSettings, [emitCsmTextSettings, 74] as TagEmitter],
    [
      TagType.DefineBitmap,
      [
        emitDefineBitmapAny,
        new Map([
          [DefineBitmapVersion.DefineBitsJpeg1, 6],
          [DefineBitmapVersion.DefineBitsLossless1, 20],
          [DefineBitmapVersion.DefineBitsJpeg2, 21],
          [DefineBitmapVersion.DefineBitsJpeg3, 35],
          [DefineBitmapVersion.DefineBitsLossless2, 36],
          [DefineBitmapVersion.DefineBitsJpeg4, 90],
        ]),
      ] as TagEmitter,
    ],
    [
      TagType.DefineButton,
      [
        emitDefineButtonAny,
        new Map([
          [ButtonVersion.Button1, 7],
          [ButtonVersion.Button2, 34],
        ]),
      ] as TagEmitter,
    ],
    [TagType.DefineButtonSound, [emitDefineButtonSound, 17] as TagEmitter],
    [TagType.DefineDynamicText, [emitDefineDynamicText, 37] as TagEmitter],
    [
      TagType.DefineFont,
      [
        emitDefineFontAny,
        new Map([
          // `Font1` is handled in `DefineGlyphFont`
          [DefineFontVersion.Font2, 48],
          [DefineFontVersion.Font3, 75],
          [DefineFontVersion.Font4, 91],
        ]),
      ] as TagEmitter,
    ],
    [TagType.DefineFontAlignZones, [emitDefineFontAlignZones, 73] as TagEmitter],
    [
      TagType.DefineFontInfo,
      [
        emitDefineFontInfoAny,
        new Map([
          [DefineFontInfoVersion.FontInfo1, 13],
          [DefineFontInfoVersion.FontInfo2, 62],
        ]),
      ] as TagEmitter,
    ],
    [TagType.DefineFontName, [emitDefineFontName, 88] as TagEmitter],
    [TagType.DefineGlyphFont, [emitDefineGlyphFont, 10] as TagEmitter],
    [TagType.DefineJpegTables, [emitDefineJpegTables, 8] as TagEmitter],
    [
      TagType.DefineMorphShape,
      [
        emitDefineMorphShapeAny,
        new Map([
          [MorphShapeVersion.MorphShape1, 46],
          [MorphShapeVersion.MorphShape2, 84],
        ]),
      ] as TagEmitter,
    ],
    [TagType.DefineSceneAndFrameLabelData, [emitDefineSceneAndFrameLabelData, 86] as TagEmitter],
    [
      TagType.DefineShape,
      [
        emitDefineShapeAny,
        new Map([
          [ShapeVersion.Shape1, 2],
          [ShapeVersion.Shape2, 22],
          [ShapeVersion.Shape3, 32],
          [ShapeVersion.Shape4, 83],
        ]),
      ] as TagEmitter,
    ],
    [TagType.DefineSound, [emitDefineSound, 14] as TagEmitter],
    [TagType.DefineSprite, [emitDefineSprite, 39] as TagEmitter],
    [
      TagType.DefineText,
      [
        emitDefineTextAny,
        new Map([
          [DefineTextVersion.DefineText1, 11],
          [DefineTextVersion.DefineText2, 33],
        ]),
      ] as TagEmitter,
    ],
    [
      TagType.DoAbc,
      [
        emitDoAbcAny,
        new Map([
          [DoAbcVersion.Abc1, 72],
          [DoAbcVersion.Abc2, 82],
        ]),
      ] as TagEmitter,
    ],
    [TagType.DoAction, [emitDoAction, 12] as TagEmitter],
    [TagType.DoInitAction, [emitDoInitAction, 59] as TagEmitter],
    [TagType.ExportAssets, [emitExportAssets, 56] as TagEmitter],
    [TagType.FileAttributes, [emitFileAttributes, 69] as TagEmitter],
    [TagType.FrameLabel, [emitFrameLabel, 43] as TagEmitter],
    [
      TagType.ImportAssets,
      [
        emitImportAssetsAny,
        new Map([
          [ImportAssetsVersion.ImportAssets1, 57],
          [ImportAssetsVersion.ImportAssets2, 71],
        ]),
      ] as TagEmitter,
    ],
    [TagType.Metadata, [emitMetadata, 77] as TagEmitter],
    [
      TagType.PlaceObject,
      [
        emitPlaceObjectAny,
        new Map([
          [PlaceObjectVersion.PlaceObject1, 4],
          [PlaceObjectVersion.PlaceObject2, 26],
          [PlaceObjectVersion.PlaceObject3, 70],
        ]),
      ] as TagEmitter,
    ],
    [TagType.Protect, [emitProtect, 24] as TagEmitter],
    [
      TagType.RemoveObject,
      [
        emitRemoveObjectAny,
        new Map([
          [RemoveObjectVersion.RemoveObject1, 5],
          [RemoveObjectVersion.RemoveObject2, 28],
        ]),
      ] as TagEmitter,
    ],
    [TagType.SetBackgroundColor, [emitSetBackgroundColor, 9] as TagEmitter],
    [TagType.StartSound, [emitStartSound, 15] as TagEmitter],
    [TagType.ShowFrame, 1],
    [TagType.SymbolClass, [emitSymbolClass, 76] as TagEmitter],
  ]);

  if (value.type === TagType.Raw) {
    byteStream.writeBytes(value.data);
    return;
  }
  if (value.type === TagType.RawBody) {
    emitTagHeader(byteStream, {code: value.code, length: value.data.length});
    byteStream.writeBytes(value.data);
    return;
  }

  const tagEmitter: TagEmitter | undefined = TAG_TYPE_TO_EMITTER.get(value.type);

  if (tagEmitter === undefined) {
    throw new incident.Incident("UnexpectedTagType", {type: value.type, typeName: TagType[value.type]});
  }

  if (typeof tagEmitter === "number") {
    emitTagHeader(byteStream, {code: tagEmitter, length: 0});
    return;
  }

  const tagStream: WritableStream = new WritableStream();
  const result: any = tagEmitter[0](tagStream, value, swfVersion);
  let code: number | Map<any, number> = tagEmitter[tagEmitter.length - 1] as any;
  if (typeof code !== "number") {
    const resolved: number | undefined = code.get(result);
    if (resolved === undefined) {
      throw new incident.Incident("UnexpectedTagVersion");
    }
    code = resolved;
  }
  emitTagHeader(byteStream, {code, length: tagStream.bytePos});
  byteStream.write(tagStream);
}

export enum DefineBitmapVersion {
  DefineBitsJpeg1,
  DefineBitsJpeg2,
  DefineBitsJpeg3,
  DefineBitsJpeg4,
  DefineBitsLossless1,
  DefineBitsLossless2,
}

export function emitDefineBitmapAny(byteStream: WritableByteStream, value: tags.DefineBitmap): DefineBitmapVersion {
  byteStream.writeUint16LE(value.id);
  switch (value.mediaType) {
    case "image/x-swf-lossless1":
      byteStream.writeBytes(value.data);
      return DefineBitmapVersion.DefineBitsLossless1;
    case "image/x-swf-lossless2":
      byteStream.writeBytes(value.data);
      return DefineBitmapVersion.DefineBitsLossless2;
    case "image/jpeg":
    case "image/gif":
    case "image/png":
      byteStream.writeBytes(value.data);
      return DefineBitmapVersion.DefineBitsJpeg2;
    case "image/x-swf-jpeg3":
      byteStream.writeBytes(value.data);
      return DefineBitmapVersion.DefineBitsJpeg3;
    case "image/x-swf-jpeg4":
      throw new Error("NotImplemented: image/x-swf-jpeg4 support");
    case "image/x-swf-partial-jpeg":
      byteStream.writeBytes(value.data);
      return DefineBitmapVersion.DefineBitsJpeg1;
    default:
      throw new incident.Incident("UnexpectedMediaType", {tag: value});
  }
}

export function emitDefineButtonAny(byteStream: WritableByteStream, value: tags.DefineButton): ButtonVersion {
  byteStream.writeUint16LE(value.id);
  const version: ButtonVersion = getMinButtonVersion(value);

  const buttonRecordStream: WritableStream = new WritableStream();
  // TODO: Select the lowest compatible `ButtonVersion`
  emitButtonRecordString(buttonRecordStream, value.records, version);

  if (version === ButtonVersion.Button1) {
    byteStream.write(buttonRecordStream);
    byteStream.writeBytes(value.actions[0].actions);
  } else {
    const flags: Uint8 = 0
      | (value.trackAsMenu ? 1 << 0 : 0);
    byteStream.writeUint8(flags);
    if (value.actions.length === 0) {
      byteStream.writeUint16LE(0);
      byteStream.write(buttonRecordStream);
    } else {
      // `+ 2` for the offset field itself
      byteStream.writeUint16LE(buttonRecordStream.bytePos + 2);
      byteStream.write(buttonRecordStream);
      emitButton2CondActionString(byteStream, value.actions);
    }
  }

  return version;
}

export function emitDefineButtonSound(byteStream: WritableByteStream, value: tags.DefineButtonSound): void {
  byteStream.writeUint16LE(value.buttonId);
  emitButtonSound(byteStream, value.overUpToIdle);
  emitButtonSound(byteStream, value.idleToOverUp);
  emitButtonSound(byteStream, value.overUpToOverDown);
  emitButtonSound(byteStream, value.overDownToOverUp);
}

export function emitCsmTextSettings(byteStream: WritableByteStream, value: tags.CsmTextSettings): void {
  byteStream.writeUint16LE(value.textId);

  const textRendererCode: Uint2 = textRendererToCode(value.renderer);
  const gridFittingCode: Uint3 = gridFittingToCode(value.fitting);
  const flags: Uint8 = 0
    // Skip bits [0, 2]
    | ((gridFittingCode & 0b111) << 3)
    | ((textRendererCode & 0b11) << 6);
  byteStream.writeUint8(flags);

  byteStream.writeFloat32LE(value.thickness);
  byteStream.writeFloat32LE(value.sharpness);
  byteStream.writeUint8(0); // Reserved
}

export enum DefineFontVersion {
  // `Font1` corresponds to `DefineGlyphFont` and is handled separately.
  Font2,
  Font3,
  Font4,
}

export function emitDefineFontAny(byteStream: WritableByteStream, value: tags.DefineFont): DefineFontVersion {
  const version: DefineFontVersion = value.emSquareSize === 20480 ? DefineFontVersion.Font3 : DefineFontVersion.Font2;

  byteStream.writeUint16LE(value.id);

  const useWideCodes: boolean = true; // `false` is deprecated since SWF6
  const offsetGlyphStream: WritableStream = new WritableStream();
  const useWideOffsets: boolean = value.glyphs !== undefined
    ? emitOffsetGlyphs(offsetGlyphStream, value.glyphs)
    : false;
  const hasLayout: boolean = value.layout !== undefined;

  const flags: Uint8 = 0
    | (value.isBold ? 1 << 0 : 0)
    | (value.isItalic ? 1 << 1 : 0)
    | (useWideCodes ? 1 << 2 : 0)
    | (useWideOffsets ? 1 << 3 : 0)
    | (value.isAnsi ? 1 << 4 : 0)
    | (value.isSmall ? 1 << 5 : 0)
    | (value.isShiftJis ? 1 << 6 : 0)
    | (hasLayout ? 1 << 7 : 0);
  byteStream.writeUint8(flags);

  emitLanguageCode(byteStream, value.language);

  const fontNameStream: WritableStream = new WritableStream();
  // TODO: Check if it should be `.writeCString` or `.writeString`
  fontNameStream.writeNulUtf8(value.fontName); // TODO: See DefineFontInfo for encoding
  byteStream.writeUint8(fontNameStream.bytePos);
  byteStream.write(fontNameStream);

  if (value.glyphs === undefined) {
    // According to Shumway:
    // > The SWF format docs doesn't say that, but the DefineFont{2,3} tag ends
    // > here for device fonts.
    byteStream.writeUint16LE(0);
    return version;
  }

  byteStream.writeUint16LE(value.glyphs.length);
  byteStream.write(offsetGlyphStream);
  // TODO: Assert codeUnits is defined (should be defined because of .glyphs)
  for (const codeUnit of value.codeUnits!) {
    // Using `.writeUint16LE` since `useWideCodes` is always `true`
    byteStream.writeUint16LE(codeUnit);
  }
  if (hasLayout) {
    emitFontLayout(byteStream, value.layout!);
  }
  return version;
}

export function emitDefineFontAlignZones(byteStream: WritableByteStream, value: tags.DefineFontAlignZones): void {
  byteStream.writeUint16LE(value.fontId);
  const bitStream: WritableBitStream = byteStream.asBitStream();
  emitCsmTableHintBits(bitStream, value.csmTableHint);
  bitStream.align();
  for (const zone of value.zones) {
    emitFontAlignmentZone(byteStream, zone);
  }
}

export enum DefineFontInfoVersion {
  FontInfo1,
  FontInfo2,
}

export function emitDefineFontInfoAny(
  byteStream: WritableByteStream,
  value: tags.DefineFontInfo,
): DefineFontInfoVersion {
  const version: DefineFontInfoVersion = value.language === LanguageCode.Auto
    ? DefineFontInfoVersion.FontInfo1
    : DefineFontInfoVersion.FontInfo2;

  byteStream.writeUint16LE(value.fontId);

  const fontNameStream: WritableStream = new WritableStream();
  fontNameStream.writeNulUtf8(value.fontName);
  byteStream.writeUint8(fontNameStream.bytePos);
  byteStream.write(fontNameStream);

  let useWideCodes: boolean = version >= DefineFontInfoVersion.FontInfo2;
  if (!useWideCodes) {
    for (const codeUnit of value.codeUnits) {
      if (codeUnit >= 256) {
        useWideCodes = true;
        break;
      }
    }
  }

  const flags: Uint8 = 0
    | (useWideCodes ? 1 << 0 : 0)
    | (value.isBold ? 1 << 1 : 0)
    | (value.isItalic ? 1 << 2 : 0)
    | (value.isAnsi ? 1 << 3 : 0)
    | (value.isShiftJis ? 1 << 4 : 0)
    | (value.isSmall ? 1 << 5 : 0);
  byteStream.writeUint8(flags);

  if (version >= DefineFontInfoVersion.FontInfo2) {
    emitLanguageCode(byteStream, value.language);
  }

  for (const codeUnit of value.codeUnits) {
    if (useWideCodes) {
      byteStream.writeUint16LE(codeUnit);
    } else {
      byteStream.writeUint8(codeUnit);
    }
  }

  return version;
}

export function emitDefineFontName(byteStream: WritableByteStream, value: tags.DefineFontName): void {
  byteStream.writeUint16LE(value.fontId);
  byteStream.writeNulUtf8(value.name);
  byteStream.writeNulUtf8(value.copyright);
}

export function emitDefineGlyphFont(byteStream: WritableByteStream, value: tags.DefineGlyphFont): void {
  byteStream.writeUint16LE(value.id);
  if (value.glyphs.length === 0) {
    return;
  }
  const firstOffset: UintSize = value.glyphs.length * 2;

  const glyphStream: WritableStream = new WritableStream();
  for (const glyph of value.glyphs) {
    byteStream.writeUint16LE(firstOffset + glyphStream.bytePos);
    emitGlyph(glyphStream, glyph);
  }

  byteStream.write(glyphStream);
}

export function emitDefineJpegTables(byteStream: WritableByteStream, value: tags.DefineJpegTables): void {
  byteStream.writeBytes(value.data);
}

export function emitDefineMorphShapeAny(
  byteStream: WritableByteStream,
  value: tags.DefineMorphShape,
): MorphShapeVersion {
  byteStream.writeUint16LE(value.id);
  emitRect(byteStream, value.bounds);
  emitRect(byteStream, value.morphBounds);

  let version: MorphShapeVersion;
  if (value.edgeBounds !== undefined && value.morphEdgeBounds !== undefined) {
    version = MorphShapeVersion.MorphShape2;
    emitRect(byteStream, value.edgeBounds);
    emitRect(byteStream, value.morphEdgeBounds);
    const flags: Uint8 = 0
      | (value.hasScalingStrokes ? 1 << 0 : 0)
      | (value.hasNonScalingStrokes ? 1 << 1 : 0);
    // Skip bits [2, 7]
    byteStream.writeUint8(flags);
  } else {
    version = MorphShapeVersion.MorphShape1;
  }

  emitMorphShape(byteStream, value.shape, version);

  return version;
}

export function emitDefineSceneAndFrameLabelData(
  byteStream: WritableByteStream,
  value: tags.DefineSceneAndFrameLabelData,
): void {
  byteStream.writeUint32Leb128(value.scenes.length);
  for (const scene of value.scenes) {
    byteStream.writeUint32Leb128(scene.offset);
    byteStream.writeNulUtf8(scene.name);
  }
  byteStream.writeUint32Leb128(value.labels.length);
  for (const label of value.labels) {
    byteStream.writeUint32Leb128(label.frame);
    byteStream.writeNulUtf8(label.name);
  }
}

function emitDefineShapeAny(byteStream: WritableByteStream, value: tags.DefineShape): ShapeVersion {
  byteStream.writeUint16LE(value.id);
  emitRect(byteStream, value.bounds);
  let shapeVersion: ShapeVersion;
  if (value.edgeBounds !== undefined) {
    shapeVersion = ShapeVersion.Shape4;
    emitRect(byteStream, value.edgeBounds);
    const flags: Uint8 = 0
      | (value.hasScalingStrokes ? 1 << 0 : 0)
      | (value.hasNonScalingStrokes ? 1 << 1 : 0)
      | (value.hasFillWinding ? 1 << 2 : 0);
    // Skip bits [3, 7]
    byteStream.writeUint8(flags);
  } else {
    shapeVersion = getMinShapeVersion(value.shape);
    // TODO: Check consistency with flags and edgeBounds
  }
  emitShape(byteStream, value.shape, shapeVersion);
  return shapeVersion;
}

// tslint:disable-next-line:cyclomatic-complexity
export function emitDefineDynamicText(byteStream: WritableByteStream, value: tags.DefineDynamicText): void {
  byteStream.writeUint16LE(value.id);
  emitRect(byteStream, value.bounds);

  const hasFont: boolean = value.fontId !== undefined && value.fontSize !== undefined;
  const hasMaxLength: boolean = value.maxLength !== undefined;
  const hasColor: boolean = value.color !== undefined;
  const hasText: boolean = value.text !== undefined;
  // TODO: Replace with `value.align !== TextAlignment.Left`
  const hasLayout: boolean = value.align !== TextAlignment.Left
    || value.marginLeft !== 0
    || value.marginRight !== 0
    || value.indent !== 0
    || value.leading !== 0;
  const hasFontClass: boolean = value.fontClass !== undefined && value.fontSize !== undefined;

  const flags: Uint16 = 0
    | (hasFont ? 1 << 0 : 0)
    | (hasMaxLength ? 1 << 1 : 0)
    | (hasColor ? 1 << 2 : 0)
    | (value.readonly ? 1 << 3 : 0)
    | (value.password ? 1 << 4 : 0)
    | (value.multiline ? 1 << 5 : 0)
    | (value.wordWrap ? 1 << 6 : 0)
    | (hasText ? 1 << 7 : 0)
    | (value.useGlyphFont ? 1 << 8 : 0)
    | (value.html ? 1 << 9 : 0)
    | (value.wasStatic ? 1 << 10 : 0)
    | (value.border ? 1 << 11 : 0)
    | (value.noSelect ? 1 << 12 : 0)
    | (hasLayout ? 1 << 13 : 0)
    | (value.autoSize ? 1 << 14 : 0)
    | (hasFontClass ? 1 << 15 : 0);
  byteStream.writeUint16LE(flags);

  if (hasFont) {
    byteStream.writeUint16LE(value.fontId!);
  }
  if (hasFontClass) {
    byteStream.writeNulUtf8(value.fontClass!);
  }
  if (hasFont || hasFontClass) {
    byteStream.writeUint16LE(value.fontSize!);
  }
  if (hasColor) {
    emitStraightSRgba8(byteStream, value.color!);
  }
  if (hasMaxLength) {
    byteStream.writeUint16LE(value.maxLength!);
  }
  if (hasLayout) {
    emitTextAlignment(byteStream, value.align);
    byteStream.writeUint16LE(value.marginLeft);
    byteStream.writeUint16LE(value.marginRight);
    byteStream.writeUint16LE(value.indent);
    byteStream.writeSint16LE(value.leading);
  }
  byteStream.writeNulUtf8(value.variableName !== undefined ? value.variableName : "");
  if (hasText) {
    byteStream.writeNulUtf8(value.text!);
  }
}

export function emitDefineSound(byteStream: WritableByteStream, value: tags.DefineSound): void {
  byteStream.writeUint16LE(value.id);

  const soundRateCode: Uint2 = soundRateToCode(value.soundRate);
  const formatCode: Uint4 = audioCodingFormatToCode(value.format);
  const flags: Uint8 = 0
    | (value.soundType === SoundType.Stereo ? 1 << 0 : 0)
    | (value.soundSize === 16 ? 1 << 1 : 0)
    | ((soundRateCode & 0b11) << 2)
    | ((formatCode & 0b1111) << 4);
  byteStream.writeUint8(flags);

  byteStream.writeUint32LE(value.sampleCount);
  byteStream.writeBytes(value.data);
}

export function emitDefineSprite(byteStream: WritableByteStream, value: tags.DefineSprite, swfVersion: Uint8): void {
  byteStream.writeUint16LE(value.id);
  byteStream.writeUint16LE(value.frameCount);
  emitTagString(byteStream, value.tags, swfVersion);
}

export enum DefineTextVersion {
  DefineText1 = 1,
  DefineText2 = 2,
}

export function emitDefineTextAny(byteStream: WritableByteStream, value: tags.DefineText): DefineTextVersion {
  byteStream.writeUint16LE(value.id);
  emitRect(byteStream, value.bounds);
  emitMatrix(byteStream, value.matrix);
  let indexBits: UintSize = 0;
  let advanceBits: UintSize = 0;
  let hasAlpha: boolean = false;
  for (const record of value.records) {
    if (record.color !== undefined && record.color.a !== 255) {
      hasAlpha = true;
    }
    for (const entry of record.entries) {
      indexBits = Math.max(indexBits, getUintBitCount(entry.index));
      advanceBits = Math.max(advanceBits, getSintBitCount(entry.advance));
    }
  }
  byteStream.writeUint8(indexBits);
  byteStream.writeUint8(advanceBits);
  emitTextRecordString(byteStream, value.records, indexBits, advanceBits, hasAlpha);
  return hasAlpha ? DefineTextVersion.DefineText2 : DefineTextVersion.DefineText1;
}

enum DoAbcVersion {
  Abc1,
  Abc2,
}

export function emitDoAbcAny(byteStream: WritableByteStream, value: tags.DoAbc): DoAbcVersion {
  let version: DoAbcVersion;
  if (value.header !== undefined) {
    version = DoAbcVersion.Abc2;
    byteStream.writeUint32LE(value.header.flags);
    byteStream.writeNulUtf8(value.header.name);
  } else {
    version = DoAbcVersion.Abc1;
  }
  byteStream.writeBytes(value.data);
  return version;
}

export function emitDoAction(byteStream: WritableByteStream, value: tags.DoAction): void {
  byteStream.writeBytes(value.actions);
}

export function emitDoInitAction(byteStream: WritableByteStream, value: tags.DoInitAction): void {
  byteStream.writeUint16LE(value.spriteId);
  byteStream.writeBytes(value.actions);
}

export function emitExportAssets(byteStream: WritableByteStream, value: tags.ExportAssets): void {
  byteStream.writeUint16LE(value.assets.length);
  for (const asset of value.assets) {
    byteStream.writeUint16LE(asset.id);
    byteStream.writeNulUtf8(asset.name);
  }
}

export function emitFileAttributes(byteStream: WritableByteStream, value: tags.FileAttributes): void {
  const flags: Uint32 = 0
    | (value.useNetwork ? 1 << 0 : 0)
    | (value.useRelativeUrls ? 1 << 1 : 0)
    | (value.noCrossDomainCaching ? 1 << 2 : 0)
    | (value.useAs3 ? 1 << 3 : 0)
    | (value.hasMetadata ? 1 << 4 : 0)
    | (value.useGpu ? 1 << 5 : 0)
    | (value.useDirectBlit ? 1 << 6 : 0);
  // Skip bits [7, 31]

  // TODO: Further investigate if a an `uint32` is really needed or if an `uint8` is enough.
  byteStream.writeUint32LE(flags);
}

export function emitFrameLabel(byteStream: WritableByteStream, value: tags.FrameLabel): void {
  byteStream.writeNulUtf8(value.name);
  if (value.isAnchor) {
    byteStream.writeUint8(1);
  }
}

export enum ImportAssetsVersion {
  ImportAssets1 = 1,
  ImportAssets2 = 2,
}

export function emitImportAssetsAny(
  byteStream: WritableByteStream,
  value: tags.ImportAssets,
  swfVersion: Uint8,
): ImportAssetsVersion {
  const version: ImportAssetsVersion = swfVersion < 8
    ? ImportAssetsVersion.ImportAssets1
    : ImportAssetsVersion.ImportAssets2;
  byteStream.writeNulUtf8(value.url);
  if (version === ImportAssetsVersion.ImportAssets2) {
    byteStream.writeUint8(1); // Must be 1 according to the spec
    byteStream.writeUint8(0); // Must be 0 according to the spec
  }
  byteStream.writeUint16LE(value.assets.length);
  for (const asset of value.assets) {
    byteStream.writeUint16LE(asset.id);
    byteStream.writeNulUtf8(asset.name);
  }
  return version;
}

export function emitMetadata(byteStream: WritableByteStream, value: tags.Metadata): void {
  byteStream.writeNulUtf8(value.metadata);
}

export enum PlaceObjectVersion {
  PlaceObject1 = 1,
  PlaceObject2 = 2,
  PlaceObject3 = 3,
}

// tslint:disable-next-line:cyclomatic-complexity
export function emitPlaceObjectAny(
  byteStream: WritableByteStream,
  value: tags.PlaceObject,
  swfVersion: UintSize,
): PlaceObjectVersion {
  const isUpdate: boolean = value.isUpdate;
  const hasCharacterId: boolean = value.characterId !== undefined;
  const hasMatrix: boolean = value.matrix !== undefined;
  const hasColorTransform: boolean = value.colorTransform !== undefined;
  const hasColorTransformWithAlpha: boolean = value.colorTransform !== undefined
    && (value.colorTransform.alphaMult.valueOf() !== 1 || value.colorTransform.alphaAdd !== 0);
  const hasRatio: boolean = value.ratio !== undefined;
  const hasName: boolean = value.name !== undefined;
  const hasClipDepth: boolean = value.clipDepth !== undefined;
  const hasClipActions: boolean = value.clipActions !== undefined;
  const hasFilters: boolean = value.filters !== undefined;
  const hasBlendMode: boolean = value.blendMode !== undefined;
  const hasCacheHint: boolean = value.bitmapCache !== undefined;
  const hasClassName: boolean = value.className !== undefined;
  const hasImage: boolean = false; // TODO: We need more context to handle images
  const hasVisibility: boolean = value.visible !== undefined;
  const hasBackgroundColor: boolean = value.backgroundColor !== undefined;

  if (hasFilters || hasBlendMode || hasCacheHint || hasClassName || hasImage || hasVisibility || hasBackgroundColor) {
    const flags: Uint16 = 0
      | (isUpdate ? 1 << 0 : 0)
      | (hasCharacterId ? 1 << 1 : 0)
      | (hasMatrix ? 1 << 2 : 0)
      | (hasColorTransform ? 1 << 3 : 0)
      | (hasRatio ? 1 << 4 : 0)
      | (hasName ? 1 << 5 : 0)
      | (hasClipDepth ? 1 << 6 : 0)
      | (hasClipActions ? 1 << 7 : 0)
      | (hasFilters ? 1 << 8 : 0)
      | (hasBlendMode ? 1 << 9 : 0)
      | (hasCacheHint ? 1 << 10 : 0)
      | (hasClassName ? 1 << 11 : 0)
      | (hasImage ? 1 << 12 : 0)
      | (hasVisibility ? 1 << 13 : 0)
      | (hasBackgroundColor ? 1 << 14 : 0);
    byteStream.writeUint16LE(flags);
    byteStream.writeUint16LE(value.depth);
    if (hasClassName) {
      byteStream.writeNulUtf8(value.className!);
    }
    if (hasCharacterId) {
      byteStream.writeUint16LE(value.characterId!);
    }
    if (hasMatrix) {
      emitMatrix(byteStream, value.matrix!);
    }
    if (hasColorTransform) {
      emitColorTransformWithAlpha(byteStream, value.colorTransform!);
    }
    if (hasRatio) {
      byteStream.writeUint16LE(value.ratio!);
    }
    if (hasName) {
      byteStream.writeNulUtf8(value.name!);
    }
    if (hasClipDepth) {
      byteStream.writeUint16LE(value.clipDepth!);
    }
    if (hasFilters) {
      emitFilterList(byteStream, value.filters!);
    }
    if (hasBlendMode) {
      emitBlendMode(byteStream, value.blendMode!);
    }
    if (hasCacheHint) {
      byteStream.writeUint8(value.bitmapCache ? 1 : 0);
    }
    if (hasVisibility) {
      byteStream.writeUint8(value.visible ? 1 : 0);
    }
    if (hasBackgroundColor) {
      emitStraightSRgba8(byteStream, value.backgroundColor!);
    }
    if (hasClipActions) {
      emitClipActionString(byteStream, value.clipActions!, swfVersion >= 6);
    }
    return PlaceObjectVersion.PlaceObject3;
  } else if (
    !hasCharacterId || !hasMatrix || isUpdate || hasColorTransformWithAlpha
    || hasRatio || hasName || hasClipDepth || hasClipActions
  ) {
    const flags: Uint8 = 0
      | (isUpdate ? 1 << 0 : 0)
      | (hasCharacterId ? 1 << 1 : 0)
      | (hasMatrix ? 1 << 2 : 0)
      | (hasColorTransform ? 1 << 3 : 0)
      | (hasRatio ? 1 << 4 : 0)
      | (hasName ? 1 << 5 : 0)
      | (hasClipDepth ? 1 << 6 : 0)
      | (hasClipActions ? 1 << 7 : 0);
    byteStream.writeUint8(flags);
    byteStream.writeUint16LE(value.depth);
    if (hasCharacterId) {
      byteStream.writeUint16LE(value.characterId!);
    }
    if (hasMatrix) {
      emitMatrix(byteStream, value.matrix!);
    }
    if (hasColorTransform) {
      emitColorTransformWithAlpha(byteStream, value.colorTransform!);
    }
    if (hasRatio) {
      byteStream.writeUint16LE(value.ratio!);
    }
    if (hasName) {
      byteStream.writeNulUtf8(value.name!);
    }
    if (hasClipDepth) {
      byteStream.writeUint16LE(value.clipDepth!);
    }
    if (hasClipActions) {
      emitClipActionString(byteStream, value.clipActions!, swfVersion >= 6);
    }
    return PlaceObjectVersion.PlaceObject2;
  } else {
    // We have (`hasCharacterId && hasMatrix && !hasColorTransformWithAlpha`)
    byteStream.writeUint16LE(value.characterId!);
    byteStream.writeUint16LE(value.depth);
    emitMatrix(byteStream, value.matrix!);
    if (hasColorTransform) {
      emitColorTransform(byteStream, value.colorTransform!);
    }
    return PlaceObjectVersion.PlaceObject1;
  }
}

export function emitProtect(byteStream: WritableByteStream, value: tags.Protect): void {
  if (value.password.length > 0) {
    byteStream.writeNulUtf8(value.password);
  }
}

export enum RemoveObjectVersion {
  RemoveObject1 = 1,
  RemoveObject2 = 2,
}

export function emitRemoveObjectAny(byteStream: WritableByteStream, value: tags.RemoveObject): RemoveObjectVersion {
  if (value.characterId !== undefined) {
    byteStream.writeUint16LE(value.characterId);
    byteStream.writeUint16LE(value.depth);
    return RemoveObjectVersion.RemoveObject1;
  } else {
    byteStream.writeUint16LE(value.depth);
    return RemoveObjectVersion.RemoveObject2;
  }
}

export function emitSetBackgroundColor(byteStream: WritableByteStream, value: tags.SetBackgroundColor): void {
  emitSRgb8(byteStream, value.color);
}

export function emitStartSound(byteStream: WritableByteStream, value: tags.StartSound): void {
  byteStream.writeUint16LE(value.soundId);
  emitSoundInfo(byteStream, value.soundInfo);
}

export function emitSymbolClass(byteStream: WritableByteStream, value: tags.SymbolClass): void {
  byteStream.writeUint16LE(value.symbols.length);
  for (const symbol of value.symbols) {
    byteStream.writeUint16LE(symbol.id);
    byteStream.writeNulUtf8(symbol.name);
  }
}
