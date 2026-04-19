use std::convert::{TryFrom, TryInto};
use std::io;

use swf_types as ast;

use crate::basic_data_types::{emit_matrix, emit_straight_s_rgba8};
use crate::bit_count::{get_i32_min_bit_count, get_u32_bit_count};
use crate::gradient::emit_morph_gradient;
use crate::io_bits::{BitsWriter, WriteBits};
use crate::primitives::{emit_le_i16, emit_le_u16, emit_le_u32, emit_u8};
use crate::shape::{cap_style_to_code, emit_edge_bits, emit_list_length, join_style_to_code};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MorphShapeVersion {
  MorphShape1,
  MorphShape2,
}

pub(crate) fn emit_morph_shape<W: io::Write>(
  writer: &mut W,
  value: &ast::MorphShape,
  version: MorphShapeVersion,
) -> io::Result<()> {
  let mut bits_writer = BitsWriter::new(Vec::new());
  let (fill_bits, line_bits) = emit_morph_shape_styles_bits(&mut bits_writer, &value.initial_styles, version)?;
  emit_morph_shape_start_record_string_bits(&mut bits_writer, &value.records, fill_bits, line_bits, version)?;
  let inner_bits_writer = bits_writer.into_inner()?;
  let start_size = inner_bits_writer.len();

  let mut bits_writer = BitsWriter::new(inner_bits_writer);

  // `0` for the style bits: there are no style changes in the end state.
  bits_writer.write_u32_bits(4, 0)?;
  bits_writer.write_u32_bits(4, 0)?;
  emit_morph_shape_end_record_string_bits(&mut bits_writer, &value.records)?;

  emit_le_u32(writer, start_size.try_into().unwrap())?;
  writer.write_all(&bits_writer.into_inner()?)
}

pub(crate) fn emit_morph_shape_styles_bits<W: WriteBits>(
  writer: &mut W,
  value: &ast::MorphShapeStyles,
  version: MorphShapeVersion,
) -> io::Result<(u32, u32)> {
  let bytes_writer = writer.write_bytes()?;
  emit_morph_fill_style_list(bytes_writer, &value.fill)?;
  emit_morph_line_style_list(bytes_writer, &value.line, version)?;
  // The max style `id` is `.len()` (and not `.len() - 1`) because `0` always
  // represents the empty style and custom styles are 1-indexed.
  let max_fill_id: u32 = u32::try_from(value.fill.len()).unwrap();
  let max_line_id: u32 = u32::try_from(value.line.len()).unwrap();
  let fill_bits: u32 = get_u32_bit_count(max_fill_id);
  let line_bits: u32 = get_u32_bit_count(max_line_id);
  writer.write_u32_bits(4, fill_bits)?;
  writer.write_u32_bits(4, line_bits)?;
  Ok((fill_bits, line_bits))
}

pub(crate) fn emit_morph_shape_start_record_string_bits<W: WriteBits>(
  writer: &mut W,
  value: &[ast::MorphShapeRecord],
  mut fill_bits: u32,
  mut line_bits: u32,
  version: MorphShapeVersion,
) -> io::Result<()> {
  for record in value {
    match record {
      ast::MorphShapeRecord::Edge(ref record) => {
        writer.write_bool_bits(true)?; // is_edge
        emit_edge_bits(
          writer,
          &ast::shape_records::Edge {
            delta: record.delta,
            control_delta: record.control_delta,
          },
        )?;
      }
      ast::MorphShapeRecord::StyleChange(ref record) => {
        writer.write_bool_bits(false)?; // is_edge
        let (next_fill_bits, next_line_bits) =
          emit_morph_style_change_bits(writer, record, fill_bits, line_bits, version)?;
        fill_bits = next_fill_bits;
        line_bits = next_line_bits;
      }
    }
  }
  writer.write_u32_bits(6, 0)
}

pub(crate) fn emit_morph_shape_end_record_string_bits<W: WriteBits>(
  writer: &mut W,
  value: &[ast::MorphShapeRecord],
) -> io::Result<()> {
  for record in value {
    match record {
      ast::MorphShapeRecord::Edge(ref record) => {
        writer.write_bool_bits(true)?; // is_edge
        emit_edge_bits(
          writer,
          &ast::shape_records::Edge {
            delta: record.morph_delta,
            control_delta: record.morph_control_delta,
          },
        )?;
      }
      ast::MorphShapeRecord::StyleChange(ref record) => {
        if record.move_to.is_none() {
          continue;
        }
        writer.write_bool_bits(false)?; // is_edge
        let flags: u8 = 0b00001; // Pure `moveTo`
        writer.write_u32_bits(5, flags.into())?;
        let morph_move_to = record.morph_move_to.unwrap();
        let bits = get_i32_min_bit_count(vec![morph_move_to.x, morph_move_to.y].into_iter());
        writer.write_u32_bits(5, bits)?;
        writer.write_i32_bits(bits, morph_move_to.x)?;
        writer.write_i32_bits(bits, morph_move_to.y)?;
      }
    }
  }
  writer.write_u32_bits(6, 0)
}

pub(crate) fn emit_morph_style_change_bits<W: WriteBits>(
  writer: &mut W,
  value: &ast::shape_records::MorphStyleChange,
  fill_bits: u32,
  line_bits: u32,
  version: MorphShapeVersion,
) -> io::Result<(u32, u32)> {
  let has_move_to = value.move_to.is_some();
  let has_new_left_fill = value.left_fill.is_some();
  let has_new_right_fill = value.right_fill.is_some();
  let has_new_line_style = value.line_style.is_some();
  let has_new_styles = value.new_styles.is_some();

  #[allow(clippy::identity_op)]
  let flags: u8 = 0
    | (if has_move_to { 1 << 0 } else { 0 })
    | (if has_new_left_fill { 1 << 1 } else { 0 })
    | (if has_new_right_fill { 1 << 2 } else { 0 })
    | (if has_new_line_style { 1 << 3 } else { 0 })
    | (if has_new_styles { 1 << 4 } else { 0 });

  assert_ne!(flags, 0);

  writer.write_u32_bits(5, flags.into())?;

  if let Some(move_to) = value.move_to {
    let bits = get_i32_min_bit_count(vec![move_to.x, move_to.y].into_iter());
    writer.write_u32_bits(5, bits)?;
    writer.write_i32_bits(bits, move_to.x)?;
    writer.write_i32_bits(bits, move_to.y)?;
  }

  if let Some(left_fill) = value.left_fill {
    writer.write_u32_bits(fill_bits, left_fill.try_into().unwrap())?;
  }
  if let Some(right_fill) = value.right_fill {
    writer.write_u32_bits(fill_bits, right_fill.try_into().unwrap())?;
  }
  if let Some(line_style) = value.line_style {
    writer.write_u32_bits(line_bits, line_style.try_into().unwrap())?;
  }

  if let Some(ref new_styles) = &value.new_styles {
    emit_morph_shape_styles_bits(writer, new_styles, version)
  } else {
    Ok((fill_bits, line_bits))
  }
}

pub(crate) fn emit_morph_fill_style_list<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &[ast::MorphFillStyle],
) -> io::Result<()> {
  emit_list_length(writer, value.len(), true)?;
  for fill_style in value {
    emit_morph_fill_style(writer, fill_style)?;
  }
  Ok(())
}

pub(crate) fn emit_morph_fill_style<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &ast::MorphFillStyle,
) -> io::Result<()> {
  match value {
    ast::MorphFillStyle::Bitmap(ref style) => {
      #[allow(clippy::identity_op)]
      let code: u8 = 0
        | (if !style.repeating { 1 << 0 } else { 0 })
        | (if !style.smoothed { 1 << 1 } else { 0 })
        | 0x40;
      emit_u8(writer, code)?;
      emit_morph_bitmap_fill(writer, style)
    }
    ast::MorphFillStyle::FocalGradient(ref style) => {
      emit_u8(writer, 0x13)?;
      emit_morph_focal_gradient_fill(writer, style)
    }
    ast::MorphFillStyle::LinearGradient(ref style) => {
      emit_u8(writer, 0x10)?;
      emit_morph_linear_gradient_fill(writer, style)
    }
    ast::MorphFillStyle::RadialGradient(ref style) => {
      emit_u8(writer, 0x12)?;
      emit_morph_radial_gradient_fill(writer, style)
    }
    ast::MorphFillStyle::Solid(ref style) => {
      emit_u8(writer, 0x00)?;
      emit_morph_solid_fill(writer, style)
    }
  }
}

pub(crate) fn emit_morph_bitmap_fill<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &ast::fill_styles::MorphBitmap,
) -> io::Result<()> {
  emit_le_u16(writer, value.bitmap_id)?;
  emit_matrix(writer, &value.matrix)?;
  emit_matrix(writer, &value.morph_matrix)
}

pub(crate) fn emit_morph_focal_gradient_fill<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &ast::fill_styles::MorphFocalGradient,
) -> io::Result<()> {
  emit_matrix(writer, &value.matrix)?;
  emit_matrix(writer, &value.morph_matrix)?;
  emit_morph_gradient(writer, &value.gradient)?;
  emit_le_i16(writer, value.focal_point.epsilons)?;
  emit_le_i16(writer, value.morph_focal_point.epsilons)
}

pub(crate) fn emit_morph_linear_gradient_fill<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &ast::fill_styles::MorphLinearGradient,
) -> io::Result<()> {
  emit_matrix(writer, &value.matrix)?;
  emit_matrix(writer, &value.morph_matrix)?;
  emit_morph_gradient(writer, &value.gradient)
}

pub(crate) fn emit_morph_radial_gradient_fill<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &ast::fill_styles::MorphRadialGradient,
) -> io::Result<()> {
  emit_matrix(writer, &value.matrix)?;
  emit_matrix(writer, &value.morph_matrix)?;
  emit_morph_gradient(writer, &value.gradient)
}

pub(crate) fn emit_morph_solid_fill<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &ast::fill_styles::MorphSolid,
) -> io::Result<()> {
  emit_straight_s_rgba8(writer, value.color)?;
  emit_straight_s_rgba8(writer, value.morph_color)
}

pub(crate) fn emit_morph_line_style_list<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &[ast::MorphLineStyle],
  version: MorphShapeVersion,
) -> io::Result<()> {
  emit_list_length(writer, value.len(), true)?;
  for line_style in value {
    if version < MorphShapeVersion::MorphShape2 {
      emit_morph_line_style1(writer, line_style)?;
    } else {
      emit_morph_line_style2(writer, line_style)?;
    }
  }
  Ok(())
}

pub(crate) fn emit_morph_line_style1<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &ast::MorphLineStyle,
) -> io::Result<()> {
  match value.fill {
    ast::MorphFillStyle::Solid(ref style) => {
      emit_le_u16(writer, value.width)?;
      emit_le_u16(writer, value.morph_width)?;
      emit_morph_solid_fill(writer, style)
    }
    _ => panic!("InvalidMorphLineStyle1Fill"),
  }
}

pub(crate) fn emit_morph_line_style2<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &ast::MorphLineStyle,
) -> io::Result<()> {
  emit_le_u16(writer, value.width)?;
  emit_le_u16(writer, value.morph_width)?;

  let has_fill = !matches!(&value.fill, ast::MorphFillStyle::Solid(_));
  let join_style_code = join_style_to_code(value.join);
  let start_cap_style_code = cap_style_to_code(value.start_cap);
  let end_cap_style_code = cap_style_to_code(value.end_cap);

  #[allow(clippy::identity_op)]
  let flags: u16 = 0
    | (if value.pixel_hinting { 1 << 0 } else { 0 })
    | (if value.no_v_scale { 1 << 1 } else { 0 })
    | (if value.no_h_scale { 1 << 2 } else { 0 })
    | (if has_fill { 1 << 3 } else { 0 })
    | ((u16::from(join_style_code) & 0b11) << 4)
    | ((u16::from(start_cap_style_code) & 0b11) << 6)
    | ((u16::from(end_cap_style_code) & 0b11) << 8)
    | (if value.no_close { 1 << 10 } else { 0 });
  // Skip bits [11, 15]
  emit_le_u16(writer, flags)?;

  if let ast::JoinStyle::Miter(miter) = value.join {
    emit_le_u16(writer, miter.limit)?;
  }

  match &value.fill {
    ast::MorphFillStyle::Solid(ref style) => emit_morph_solid_fill(writer, style),
    style => emit_morph_fill_style(writer, style),
  }
}
