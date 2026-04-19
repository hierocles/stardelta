use std::convert::TryInto;
use std::io;

use swf_types as ast;

use crate::basic_data_types::{emit_rect, emit_s_rgb8, emit_straight_s_rgba8};
use crate::io_bits::{BitsWriter, WriteBits};
use crate::primitives::{emit_le_f16, emit_le_i16, emit_le_u16, emit_le_u32, emit_u8};
use crate::shape::emit_glyph;

// TODO: Remove unused variants (`dead_code` should not be allowed)
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum DefineFontVersion {
  // `Font1` corresponds to `DefineGlyphFont` and is handled separately.
  Font2,
  Font3,
  Font4,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum DefineFontInfoVersion {
  FontInfo1,
  FontInfo2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum DefineTextVersion {
  Text1,
  Text2,
}

pub(crate) fn csm_table_hint_to_code(value: ast::text::CsmTableHint) -> u8 {
  match value {
    ast::text::CsmTableHint::Thin => 0,
    ast::text::CsmTableHint::Medium => 1,
    ast::text::CsmTableHint::Thick => 2,
  }
}

pub(crate) fn emit_language_code<W: io::Write>(writer: &mut W, value: ast::LanguageCode) -> io::Result<()> {
  let code: u8 = match value {
    ast::LanguageCode::Auto => 0,
    ast::LanguageCode::Latin => 1,
    ast::LanguageCode::Japanese => 2,
    ast::LanguageCode::Korean => 3,
    ast::LanguageCode::SimplifiedChinese => 4,
    ast::LanguageCode::TraditionalChinese => 5,
  };
  emit_u8(writer, code)
}

pub(crate) fn grid_fitting_to_code(value: ast::text::GridFitting) -> u8 {
  match value {
    ast::text::GridFitting::None => 0,
    ast::text::GridFitting::Pixel => 1,
    ast::text::GridFitting::SubPixel => 2,
  }
}

pub(crate) fn text_renderer_to_code(value: ast::text::TextRenderer) -> u8 {
  match value {
    ast::text::TextRenderer::Advanced => 1,
    ast::text::TextRenderer::Normal => 0,
  }
}

pub(crate) fn emit_text_record_string<W: io::Write>(
  writer: &mut W,
  value: &[ast::text::TextRecord],
  index_bits: u32,
  advance_bits: u32,
  with_alpha: bool,
) -> io::Result<()> {
  for record in value {
    emit_text_record(writer, record, index_bits, advance_bits, with_alpha)?;
  }
  emit_u8(writer, 0)
}

pub(crate) fn emit_text_record<W: io::Write>(
  writer: &mut W,
  value: &ast::text::TextRecord,
  index_bits: u32,
  advance_bits: u32,
  with_alpha: bool,
) -> io::Result<()> {
  let has_offset_x = value.offset_x != 0;
  let has_offset_y = value.offset_y != 0;
  let has_color = value.color.is_some();
  let has_font = value.font_id.is_some() && value.font_size.is_some();

  #[allow(clippy::identity_op)]
  let flags: u8 = 0
    | (if has_offset_x { 1 << 0 } else { 0 })
    | (if has_offset_y { 1 << 1 } else { 0 })
    | (if has_color { 1 << 2 } else { 0 })
    | (if has_font { 1 << 3 } else { 0 })
    // Skip bits [4, 6]
    | (1 << 7); // Bit 7 must be set (TextRecordType)
  emit_u8(writer, flags)?;

  if let Some(font_id) = value.font_id {
    assert!(has_font);
    emit_le_u16(writer, font_id)?;
  }
  if let Some(color) = value.color {
    if with_alpha {
      emit_straight_s_rgba8(writer, color)?;
    } else {
      assert!(color.a == u8::max_value());
      emit_s_rgb8(
        writer,
        ast::SRgb8 {
          r: color.r,
          g: color.g,
          b: color.b,
        },
      )?;
    }
  }
  if has_offset_x {
    emit_le_i16(writer, value.offset_x)?;
  }
  if has_offset_y {
    emit_le_i16(writer, value.offset_y)?;
  }
  if let Some(font_size) = value.font_size {
    assert!(has_font);
    emit_le_u16(writer, font_size)?;
  }
  emit_u8(writer, value.entries.len().try_into().unwrap())?;
  let mut bits_writer = BitsWriter::new(Vec::new());
  for entry in &value.entries {
    bits_writer.write_u32_bits(index_bits, entry.index.try_into().unwrap())?;
    bits_writer.write_i32_bits(advance_bits, entry.advance)?;
  }
  writer.write_all(&bits_writer.into_inner()?)
}

pub(crate) fn emit_font_alignment_zone<W: io::Write>(
  writer: &mut W,
  value: &ast::text::FontAlignmentZone,
) -> io::Result<()> {
  assert!(value.data.len() < 256);

  emit_u8(writer, value.data.len().try_into().unwrap())?;
  for zone_data in &value.data {
    emit_le_f16(writer, zone_data.origin)?;
    emit_le_f16(writer, zone_data.size)?;
  }
  #[allow(clippy::identity_op)]
  let flags: u8 = 0 | (if value.has_x { 1 << 0 } else { 0 }) | (if value.has_y { 1 << 1 } else { 0 });
  // Skip bits [2, 7]
  emit_u8(writer, flags)
}

pub(crate) fn emit_offset_glyphs<W: io::Write>(writer: &mut W, value: &[ast::Glyph]) -> io::Result<bool> {
  let mut end_offsets: Vec<usize> = Vec::with_capacity(value.len());
  let mut glyph_writer: Vec<u8> = Vec::new();
  for glyph in value {
    emit_glyph(&mut glyph_writer, glyph)?;
    end_offsets.push(glyph_writer.len());
  }

  let offset_table_len = end_offsets.len() + 1;
  let short_offset_table_size = offset_table_len * std::mem::size_of::<u16>();
  let max_offset_with_short_table = short_offset_table_size + glyph_writer.len();

  let use_wide_offsets = max_offset_with_short_table > usize::from(u16::max_value());

  if use_wide_offsets {
    let wide_offset_table_size = offset_table_len * std::mem::size_of::<u32>();

    emit_le_u32(writer, wide_offset_table_size.try_into().unwrap())?;
    for end_offset in end_offsets {
      emit_le_u32(writer, (wide_offset_table_size + end_offset).try_into().unwrap())?;
    }
  } else {
    emit_le_u16(writer, short_offset_table_size.try_into().unwrap())?;
    for end_offset in end_offsets {
      emit_le_u16(writer, (short_offset_table_size + end_offset).try_into().unwrap())?;
    }
  }

  writer.write_all(&glyph_writer)?;

  Ok(use_wide_offsets)
}

pub(crate) fn emit_font_layout<W: io::Write>(writer: &mut W, value: &ast::text::FontLayout) -> io::Result<()> {
  emit_le_u16(writer, value.ascent)?;
  emit_le_u16(writer, value.descent)?;
  emit_le_u16(writer, value.leading)?;
  for advance in &value.advances {
    emit_le_u16(writer, *advance)?;
  }
  for bound in &value.bounds {
    emit_rect(writer, bound)?;
  }
  emit_le_u16(writer, value.kerning.len().try_into().unwrap())?;
  for kerning_record in &value.kerning {
    emit_kerning_record(writer, kerning_record)?;
  }
  Ok(())
}

pub(crate) fn emit_kerning_record<W: io::Write>(writer: &mut W, value: &ast::text::KerningRecord) -> io::Result<()> {
  emit_le_u16(writer, value.left)?;
  emit_le_u16(writer, value.right)?;
  emit_le_i16(writer, value.adjustment)
}

pub(crate) fn emit_text_alignment<W: io::Write>(writer: &mut W, value: ast::text::TextAlignment) -> io::Result<()> {
  let code: u8 = match value {
    ast::text::TextAlignment::Center => 2,
    ast::text::TextAlignment::Justify => 3,
    ast::text::TextAlignment::Left => 0,
    ast::text::TextAlignment::Right => 1,
  };
  emit_u8(writer, code)
}
