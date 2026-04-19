use std::convert::TryInto;
use std::io;

use swf_types as ast;

use crate::primitives::{emit_le_u16, emit_le_u32, emit_u8};

pub(crate) fn audio_coding_format_to_code(value: ast::AudioCodingFormat) -> u8 {
  match value {
    ast::AudioCodingFormat::UncompressedNativeEndian => 0,
    ast::AudioCodingFormat::Adpcm => 1,
    ast::AudioCodingFormat::Mp3 => 2,
    ast::AudioCodingFormat::UncompressedLittleEndian => 3,
    ast::AudioCodingFormat::Nellymoser16 => 4,
    ast::AudioCodingFormat::Nellymoser8 => 5,
    ast::AudioCodingFormat::Nellymoser => 6,
    ast::AudioCodingFormat::Speex => 11,
  }
}

pub(crate) fn sound_rate_to_code(value: ast::SoundRate) -> u8 {
  match value {
    ast::SoundRate::SoundRate5500 => 0,
    ast::SoundRate::SoundRate11000 => 1,
    ast::SoundRate::SoundRate22000 => 2,
    ast::SoundRate::SoundRate44000 => 3,
  }
}

pub(crate) fn emit_sound_info<W: io::Write>(writer: &mut W, value: &ast::SoundInfo) -> io::Result<()> {
  let has_in_point = value.in_point.is_some();
  let has_out_point = value.out_point.is_some();
  let has_loops = value.loop_count.is_some();
  let has_envelope = value.envelope_records.is_some();
  let sync_no_multiple = value.sync_no_multiple;
  let sync_stop = value.sync_stop;

  #[allow(clippy::identity_op)]
  let flags: u8 = 0
    | (if has_in_point { 1 << 0 } else { 0 })
    | (if has_out_point { 1 << 1 } else { 0 })
    | (if has_loops { 1 << 2 } else { 0 })
    | (if has_envelope { 1 << 3 } else { 0 })
    | (if sync_no_multiple { 1 << 4 } else { 0 })
    | (if sync_stop { 1 << 5 } else { 0 });
  // Skip bits [6, 7]
  emit_u8(writer, flags)?;

  if let Some(in_point) = value.in_point {
    emit_le_u32(writer, in_point)?;
  }
  if let Some(out_point) = value.out_point {
    emit_le_u32(writer, out_point)?;
  }
  if let Some(loop_count) = value.loop_count {
    emit_le_u16(writer, loop_count)?;
  }
  if let Some(ref envelope) = &value.envelope_records {
    emit_sound_envelope(writer, envelope)?;
  }

  Ok(())
}

pub(crate) fn emit_sound_envelope<W: io::Write>(writer: &mut W, value: &[ast::SoundEnvelope]) -> io::Result<()> {
  emit_u8(writer, value.len().try_into().unwrap())?;
  for record in value {
    emit_le_u32(writer, record.pos44)?;
    emit_le_u16(writer, record.left_level)?;
    emit_le_u16(writer, record.right_level)?;
  }

  Ok(())
}
