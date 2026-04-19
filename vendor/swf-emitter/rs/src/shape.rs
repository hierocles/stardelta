use std::cmp::max;
use std::convert::{TryFrom, TryInto};
use std::io;

use swf_types as ast;

use crate::basic_data_types::{emit_matrix, emit_s_rgb8, emit_straight_s_rgba8};
use crate::bit_count::{get_i32_min_bit_count, get_u32_bit_count};
use crate::gradient::emit_gradient;
use crate::io_bits::{BitsWriter, WriteBits};
use crate::primitives::{emit_le_i16, emit_le_u16, emit_u8};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShapeVersion {
  Shape1,
  Shape2,
  Shape3,
  Shape4,
}

pub(crate) fn emit_glyph<W: io::Write>(writer: &mut W, value: &ast::Glyph) -> io::Result<()> {
  let mut bits_writer = BitsWriter::new(Vec::new());
  emit_glyph_bits(&mut bits_writer, value)?;
  writer.write_all(&bits_writer.into_inner()?)
}

pub(crate) fn emit_glyph_bits<W: WriteBits>(writer: &mut W, value: &ast::Glyph) -> io::Result<()> {
  // TODO: Check how to determine the bit count (scan records?)
  let fill_bits: u32 = 1; // 2 styles (empty and filled) -> 1 bit
  let line_bits: u32 = 0; // no line styles
  writer.write_u32_bits(4, fill_bits)?;
  writer.write_u32_bits(4, line_bits)?;
  // TODO: Check which shape version to use
  emit_shape_record_string_bits(writer, &value.records, fill_bits, line_bits, ShapeVersion::Shape1)
}

pub(crate) fn emit_shape<W: io::Write>(writer: &mut W, value: &ast::Shape, version: ShapeVersion) -> io::Result<()> {
  let mut bits_writer = BitsWriter::new(Vec::new());
  emit_shape_bits(&mut bits_writer, value, version)?;
  writer.write_all(&bits_writer.into_inner()?)
}

pub(crate) fn emit_shape_bits<W: WriteBits>(
  writer: &mut W,
  value: &ast::Shape,
  version: ShapeVersion,
) -> io::Result<()> {
  let (fill_bits, line_bits) = emit_shape_styles_bits(writer, &value.initial_styles, version)?;
  emit_shape_record_string_bits(writer, &value.records, fill_bits, line_bits, version)
}

pub(crate) fn emit_shape_styles_bits<W: WriteBits>(
  writer: &mut W,
  value: &ast::ShapeStyles,
  version: ShapeVersion,
) -> io::Result<(u32, u32)> {
  let bytes_writer = writer.write_bytes()?;
  emit_fill_style_list(bytes_writer, &value.fill, version)?;
  emit_line_style_list(bytes_writer, &value.line, version)?;
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

pub(crate) fn emit_shape_record_string_bits<W: WriteBits>(
  writer: &mut W,
  value: &[ast::ShapeRecord],
  mut fill_bits: u32,
  mut line_bits: u32,
  version: ShapeVersion,
) -> io::Result<()> {
  for record in value {
    match record {
      ast::ShapeRecord::Edge(ref record) => {
        writer.write_bool_bits(true)?; // is_edge
        emit_edge_bits(writer, record)?;
      }
      ast::ShapeRecord::StyleChange(ref record) => {
        writer.write_bool_bits(false)?; // is_edge
        let (next_fill_bits, next_line_bits) = emit_style_change_bits(writer, record, fill_bits, line_bits, version)?;
        fill_bits = next_fill_bits;
        line_bits = next_line_bits;
      }
    }
  }
  writer.write_u32_bits(6, 0)
}

pub(crate) fn emit_edge_bits<W: WriteBits>(writer: &mut W, value: &ast::shape_records::Edge) -> io::Result<()> {
  if let Some(control_delta) = value.control_delta {
    writer.write_bool_bits(false)?; // is_straight
    let anchor_delta = ast::Vector2D {
      x: value.delta.x - control_delta.x,
      y: value.delta.y - control_delta.y,
    };
    let bits =
      get_i32_min_bit_count(vec![control_delta.x, control_delta.y, anchor_delta.x, anchor_delta.y].into_iter());
    let bits = 2 + bits.saturating_sub(2);
    writer.write_u32_bits(4, bits - 2)?;
    writer.write_i32_bits(bits, control_delta.x)?;
    writer.write_i32_bits(bits, control_delta.y)?;
    writer.write_i32_bits(bits, anchor_delta.x)?;
    writer.write_i32_bits(bits, anchor_delta.y)?;
  } else {
    writer.write_bool_bits(true)?; // is_straight
    let bits = get_i32_min_bit_count(vec![value.delta.x, value.delta.y].into_iter());
    let bits = 2 + bits.saturating_sub(2);
    writer.write_u32_bits(4, bits - 2)?;
    let is_diagonal = value.delta.x != 0 && value.delta.y != 0;
    writer.write_bool_bits(is_diagonal)?;
    if is_diagonal {
      writer.write_i32_bits(bits, value.delta.x)?;
      writer.write_i32_bits(bits, value.delta.y)?;
    } else {
      let is_vertical = value.delta.x == 0;
      writer.write_bool_bits(is_vertical)?;
      if is_vertical {
        writer.write_i32_bits(bits, value.delta.y)?;
      } else {
        writer.write_i32_bits(bits, value.delta.x)?;
      }
    }
  }
  Ok(())
}

pub(crate) fn emit_style_change_bits<W: WriteBits>(
  writer: &mut W,
  value: &ast::shape_records::StyleChange,
  fill_bits: u32,
  line_bits: u32,
  version: ShapeVersion,
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
    emit_shape_styles_bits(writer, new_styles, version)
  } else {
    Ok((fill_bits, line_bits))
  }
}

pub(crate) fn emit_list_length<W: io::Write + ?Sized>(
  writer: &mut W,
  value: usize,
  support_extended: bool,
) -> io::Result<()> {
  if !support_extended {
    assert!(value <= 0xff);
    emit_u8(writer, value.try_into().unwrap())
  } else {
    assert!(value <= 0xffff);
    if value < 0xff {
      emit_u8(writer, value.try_into().unwrap())
    } else {
      emit_u8(writer, 0xff)?;
      emit_le_u16(writer, value.try_into().unwrap())
    }
  }
}

pub(crate) fn emit_fill_style_list<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &[ast::FillStyle],
  version: ShapeVersion,
) -> io::Result<()> {
  emit_list_length(writer, value.len(), version >= ShapeVersion::Shape2)?;
  for fill_style in value {
    emit_fill_style(writer, fill_style, version >= ShapeVersion::Shape3)?;
  }
  Ok(())
}

pub(crate) fn emit_fill_style<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &ast::FillStyle,
  with_alpha: bool,
) -> io::Result<()> {
  match value {
    ast::FillStyle::Bitmap(ref style) => {
      #[allow(clippy::identity_op)]
      let code: u8 = 0
        | (if !style.repeating { 1 << 0 } else { 0 })
        | (if !style.smoothed { 1 << 1 } else { 0 }) | 0x40;
      emit_u8(writer, code)?;
      emit_bitmap_fill(writer, style)
    }
    ast::FillStyle::FocalGradient(ref style) => {
      emit_u8(writer, 0x13)?;
      emit_focal_gradient_fill(writer, style, with_alpha)
    }
    ast::FillStyle::LinearGradient(ref style) => {
      emit_u8(writer, 0x10)?;
      emit_linear_gradient_fill(writer, style, with_alpha)
    }
    ast::FillStyle::RadialGradient(ref style) => {
      emit_u8(writer, 0x12)?;
      emit_radial_gradient_fill(writer, style, with_alpha)
    }
    ast::FillStyle::Solid(ref style) => {
      emit_u8(writer, 0x00)?;
      emit_solid_fill(writer, style, with_alpha)
    }
  }
}

pub(crate) fn emit_bitmap_fill<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &ast::fill_styles::Bitmap,
) -> io::Result<()> {
  emit_le_u16(writer, value.bitmap_id)?;
  emit_matrix(writer, &value.matrix)
}

pub(crate) fn emit_focal_gradient_fill<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &ast::fill_styles::FocalGradient,
  with_alpha: bool,
) -> io::Result<()> {
  emit_matrix(writer, &value.matrix)?;
  emit_gradient(writer, &value.gradient, with_alpha)?;
  emit_le_i16(writer, value.focal_point.epsilons)
}

pub(crate) fn emit_linear_gradient_fill<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &ast::fill_styles::LinearGradient,
  with_alpha: bool,
) -> io::Result<()> {
  emit_matrix(writer, &value.matrix)?;
  emit_gradient(writer, &value.gradient, with_alpha)
}

pub(crate) fn emit_radial_gradient_fill<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &ast::fill_styles::RadialGradient,
  with_alpha: bool,
) -> io::Result<()> {
  emit_matrix(writer, &value.matrix)?;
  emit_gradient(writer, &value.gradient, with_alpha)
}

pub(crate) fn emit_solid_fill<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &ast::fill_styles::Solid,
  with_alpha: bool,
) -> io::Result<()> {
  if with_alpha {
    emit_straight_s_rgba8(writer, value.color)
  } else {
    assert!(value.color.a == u8::max_value());
    emit_s_rgb8(
      writer,
      ast::SRgb8 {
        r: value.color.r,
        g: value.color.g,
        b: value.color.b,
      },
    )
  }
}

pub(crate) fn emit_line_style_list<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &[ast::LineStyle],
  version: ShapeVersion,
) -> io::Result<()> {
  emit_list_length(writer, value.len(), version >= ShapeVersion::Shape2)?;
  for line_style in value {
    if version < ShapeVersion::Shape4 {
      emit_line_style1(writer, line_style, version >= ShapeVersion::Shape3)?;
    } else {
      emit_line_style2(writer, line_style)?;
    }
  }
  Ok(())
}

pub(crate) fn emit_line_style1<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &ast::LineStyle,
  with_alpha: bool,
) -> io::Result<()> {
  match value.fill {
    ast::FillStyle::Solid(ref style) => {
      emit_le_u16(writer, value.width)?;
      emit_solid_fill(writer, style, with_alpha)
    }
    _ => panic!("InvalidLineStyle1Fill"),
  }
}

pub(crate) fn emit_line_style2<W: io::Write + ?Sized>(writer: &mut W, value: &ast::LineStyle) -> io::Result<()> {
  emit_le_u16(writer, value.width)?;

  let has_fill = !matches!(&value.fill, ast::FillStyle::Solid(_));
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
    ast::FillStyle::Solid(ref style) => emit_solid_fill(writer, style, true),
    style => emit_fill_style(writer, style, true),
  }
}

pub(crate) fn join_style_to_code(value: ast::JoinStyle) -> u8 {
  match value {
    ast::JoinStyle::Bevel => 1,
    ast::JoinStyle::Round => 0,
    ast::JoinStyle::Miter(_) => 2,
  }
}

pub(crate) fn cap_style_to_code(value: ast::CapStyle) -> u8 {
  match value {
    ast::CapStyle::None => 1,
    ast::CapStyle::Round => 0,
    ast::CapStyle::Square => 2,
  }
}

pub(crate) fn get_min_shape_version(value: &ast::Shape) -> ShapeVersion {
  value.records.iter().fold(
    get_shape_styles_min_shape_version(&value.initial_styles),
    |acc, record| match record {
      ast::ShapeRecord::StyleChange(ref record) => match &record.new_styles {
        Some(ref styles) => max(acc, get_shape_styles_min_shape_version(styles)),
        _ => acc,
      },
      _ => acc,
    },
  )
}

pub(crate) fn get_shape_styles_min_shape_version(value: &ast::ShapeStyles) -> ShapeVersion {
  max(
    get_fill_style_list_min_shape_version(&value.fill),
    get_line_style_list_min_shape_version(&value.line),
  )
}

pub(crate) fn get_fill_style_list_min_shape_version(value: &[ast::FillStyle]) -> ShapeVersion {
  value.iter().map(get_fill_style_min_shape_version).fold(
    if value.len() < 0xff {
      ShapeVersion::Shape1
    } else {
      ShapeVersion::Shape2
    },
    max,
  )
}

pub(crate) fn get_fill_style_min_shape_version(value: &ast::FillStyle) -> ShapeVersion {
  let has_alpha = match value {
    ast::FillStyle::Solid(ref style) => style.color.a != u8::max_value(),
    ast::FillStyle::FocalGradient(ref style) => style.gradient.colors.iter().any(|cs| cs.color.a != u8::max_value()),
    ast::FillStyle::LinearGradient(ref style) => style.gradient.colors.iter().any(|cs| cs.color.a != u8::max_value()),
    ast::FillStyle::RadialGradient(ref style) => style.gradient.colors.iter().any(|cs| cs.color.a != u8::max_value()),
    _ => false,
  };

  if has_alpha {
    ShapeVersion::Shape3
  } else {
    ShapeVersion::Shape1
  }
}

pub(crate) fn get_line_style_list_min_shape_version(value: &[ast::LineStyle]) -> ShapeVersion {
  value.iter().map(get_line_style_min_shape_version).fold(
    if value.len() < 0xff {
      ShapeVersion::Shape1
    } else {
      ShapeVersion::Shape2
    },
    max,
  )
}

pub(crate) fn get_line_style_min_shape_version(value: &ast::LineStyle) -> ShapeVersion {
  let is_solid_fill = !matches!(&value.fill, ast::FillStyle::Solid(_));
  let is_line_style2 = value.start_cap != ast::CapStyle::Round
    || value.end_cap != ast::CapStyle::Round
    || value.join != ast::JoinStyle::Round
    || value.no_h_scale
    || value.no_v_scale
    || value.no_close
    || value.pixel_hinting
    || !is_solid_fill;

  if is_line_style2 {
    ShapeVersion::Shape4
  } else {
    match &value.fill {
      ast::FillStyle::Solid(ref style) if style.color.a != u8::max_value() => ShapeVersion::Shape3,
      _ => ShapeVersion::Shape1,
    }
  }
}
