import { WritableByteStream } from "@open-flash/stream";
import incident from "incident";
import { Uint2, Uint4, Uint8 } from "semantic-types";
import { AudioCodingFormat } from "swf-types/sound/audio-coding-format";
import { SoundEnvelope } from "swf-types/sound/sound-envelope";
import { SoundInfo } from "swf-types/sound/sound-info";
import { SoundRate } from "swf-types/sound/sound-rate";

export function audioCodingFormatToCode(value: AudioCodingFormat): Uint4 {
  switch (value) {
    case AudioCodingFormat.UncompressedNativeEndian:
      return 0;
    case AudioCodingFormat.Adpcm:
      return 1;
    case AudioCodingFormat.Mp3:
      return 2;
    case AudioCodingFormat.UncompressedLittleEndian:
      return 3;
    case AudioCodingFormat.Nellymoser16:
      return 4;
    case AudioCodingFormat.Nellymoser8:
      return 5;
    case AudioCodingFormat.Nellymoser:
      return 6;
    case AudioCodingFormat.Speex:
      return 11;
    default:
      throw new incident.Incident("UnexpectedAudioCodingFormat");
  }
}

export function soundRateToCode(value: SoundRate): Uint2 {
  switch (value) {
    case 5500:
      return 0;
    case 11000:
      return 1;
    case 22000:
      return 2;
    case 44000:
      return 3;
    default:
      throw new incident.Incident("UnexpectedSoundRate");
  }
}

export function emitSoundInfo(byteStream: WritableByteStream, value: SoundInfo): void {
  const hasInPoint: boolean = value.inPoint !== undefined;
  const hasOutPoint: boolean = value.outPoint !== undefined;
  const hasLoops: boolean = value.loopCount !== undefined;
  const hasEnvelope: boolean = value.envelopeRecords !== undefined;
  const syncNoMultiple: boolean = value.syncNoMultiple;
  const syncStop: boolean = value.syncStop;

  const flags: Uint8 = 0
    | (hasInPoint ? 1 << 0 : 0)
    | (hasOutPoint ? 1 << 1 : 0)
    | (hasLoops ? 1 << 2 : 0)
    | (hasEnvelope ? 1 << 3 : 0)
    | (syncNoMultiple ? 1 << 4 : 0)
    | (syncStop ? 1 << 5 : 0);
  // Skip bits [6, 7]
  byteStream.writeUint8(flags);

  if (value.inPoint !== undefined) {
    byteStream.writeUint32LE(value.inPoint);
  }
  if (value.outPoint !== undefined) {
    byteStream.writeUint32LE(value.outPoint);
  }
  if (value.loopCount !== undefined) {
    byteStream.writeUint16LE(value.loopCount);
  }
  if (value.envelopeRecords !== undefined) {
    emitSoundEnvelope(byteStream, value.envelopeRecords);
  }
}

export function emitSoundEnvelope(
  byteStream: WritableByteStream,
  value: readonly SoundEnvelope[],
): void {
  byteStream.writeUint8(value.length);
  for (const record of value) {
    byteStream.writeUint32LE(record.pos44);
    byteStream.writeUint16LE(record.leftLevel);
    byteStream.writeUint16LE(record.rightLevel);
  }
}
