use std::convert::TryInto;
use std::io;

use swf_fixed::{Sfixed16P16, Sfixed8P8};
use swf_types as ast;

use crate::bit_count::get_i32_min_bit_count;
use crate::io_bits::{BitsWriter, WriteBits};
use crate::primitives::emit_u8;

/// Emits a null-terminated string.
pub fn emit_c_string<W: io::Write>(writer: &mut W, value: &str) -> io::Result<()> {
  writer.write_all(value.as_bytes())?;
  writer.write_all(&[0])
}

pub fn emit_leb128_u32<W: io::Write>(writer: &mut W, mut value: u32) -> io::Result<()> {
  if value == 0 {
    return emit_u8(writer, 0);
  }
  while value != 0 {
    let mut next_byte: u8 = (value & 0x7f).try_into().unwrap();
    value >>= 7;
    if value != 0 {
      next_byte |= 0x80;
    }
    emit_u8(writer, next_byte)?;
  }
  Ok(())
}

pub fn emit_rect<W: io::Write>(writer: &mut W, value: &ast::Rect) -> io::Result<()> {
  let mut bits_writer = BitsWriter::new(Vec::new());
  emit_rect_bits(&mut bits_writer, value)?;
  writer.write_all(&bits_writer.into_inner()?)
}

pub fn emit_rect_bits<W: WriteBits>(writer: &mut W, value: &ast::Rect) -> io::Result<()> {
  let bits = get_i32_min_bit_count(vec![value.x_min, value.x_max, value.y_min, value.y_max].into_iter());
  writer.write_u32_bits(5, bits)?;
  writer.write_i32_bits(bits, value.x_min)?;
  writer.write_i32_bits(bits, value.x_max)?;
  writer.write_i32_bits(bits, value.y_min)?;
  writer.write_i32_bits(bits, value.y_max)
}

pub fn emit_s_rgb8<W: io::Write + ?Sized>(writer: &mut W, value: ast::SRgb8) -> io::Result<()> {
  emit_u8(writer, value.r)?;
  emit_u8(writer, value.g)?;
  emit_u8(writer, value.b)
}

pub fn emit_straight_s_rgba8<W: io::Write + ?Sized>(writer: &mut W, value: ast::StraightSRgba8) -> io::Result<()> {
  emit_u8(writer, value.r)?;
  emit_u8(writer, value.g)?;
  emit_u8(writer, value.b)?;
  emit_u8(writer, value.a)
}

pub fn emit_matrix<W: io::Write + ?Sized>(writer: &mut W, value: &ast::Matrix) -> io::Result<()> {
  let mut bits_writer = BitsWriter::new(Vec::new());
  emit_matrix_bits(&mut bits_writer, value)?;
  writer.write_all(&bits_writer.into_inner()?)
}

pub fn emit_matrix_bits<W: WriteBits>(writer: &mut W, value: &ast::Matrix) -> io::Result<()> {
  if value.scale_x == Sfixed16P16::ONE && value.scale_y == Sfixed16P16::ONE {
    writer.write_bool_bits(false)?;
  } else {
    writer.write_bool_bits(true)?;
    let bits = get_i32_min_bit_count(vec![value.scale_x.epsilons, value.scale_y.epsilons].into_iter());
    writer.write_u32_bits(5, bits)?;
    writer.write_i32_bits(bits, value.scale_x.epsilons)?;
    writer.write_i32_bits(bits, value.scale_y.epsilons)?;
  }

  if value.rotate_skew0 == Sfixed16P16::ZERO && value.rotate_skew1 == Sfixed16P16::ZERO {
    writer.write_bool_bits(false)?;
  } else {
    writer.write_bool_bits(true)?;
    let bits = get_i32_min_bit_count(vec![value.rotate_skew0.epsilons, value.rotate_skew1.epsilons].into_iter());
    writer.write_u32_bits(5, bits)?;
    writer.write_i32_bits(bits, value.rotate_skew0.epsilons)?;
    writer.write_i32_bits(bits, value.rotate_skew1.epsilons)?;
  }

  {
    let bits = get_i32_min_bit_count(vec![value.translate_x, value.translate_y].into_iter());
    writer.write_u32_bits(5, bits)?;
    writer.write_i32_bits(bits, value.translate_x)?;
    writer.write_i32_bits(bits, value.translate_y)?;
  }

  Ok(())
}

pub fn emit_color_transform<W: io::Write>(writer: &mut W, value: &ast::ColorTransform) -> io::Result<()> {
  let mut bits_writer = BitsWriter::new(Vec::new());
  emit_color_transform_bits(&mut bits_writer, value)?;
  writer.write_all(&bits_writer.into_inner()?)
}

pub fn emit_color_transform_bits<W: WriteBits>(writer: &mut W, value: &ast::ColorTransform) -> io::Result<()> {
  let has_add = value.red_add != 0 || value.green_add != 0 || value.blue_add != 0;
  let has_mult =
    value.red_mult != Sfixed8P8::ONE || value.green_mult != Sfixed8P8::ONE || value.blue_mult != Sfixed8P8::ONE;

  let mut to_write: Vec<i32> = Vec::new();
  if has_mult {
    to_write.extend_from_slice(&[
      value.red_mult.epsilons.into(),
      value.green_mult.epsilons.into(),
      value.blue_mult.epsilons.into(),
    ]);
  }
  if has_add {
    to_write.extend_from_slice(&[value.red_add.into(), value.green_add.into(), value.blue_add.into()]);
  }

  let bits = get_i32_min_bit_count(to_write.clone().into_iter());

  writer.write_bool_bits(has_add)?;
  writer.write_bool_bits(has_mult)?;
  writer.write_u32_bits(4, bits)?;

  for value in to_write {
    writer.write_i32_bits(bits, value)?;
  }

  Ok(())
}

pub fn emit_color_transform_with_alpha<W: io::Write>(
  writer: &mut W,
  value: &ast::ColorTransformWithAlpha,
) -> io::Result<()> {
  let mut bits_writer = BitsWriter::new(Vec::new());
  emit_color_transform_with_alpha_bits(&mut bits_writer, value)?;
  writer.write_all(&bits_writer.into_inner()?)
}

pub fn emit_color_transform_with_alpha_bits<W: WriteBits>(
  writer: &mut W,
  value: &ast::ColorTransformWithAlpha,
) -> io::Result<()> {
  let has_add = value.red_add != 0 || value.green_add != 0 || value.blue_add != 0 || value.alpha_add != 0;
  let has_mult = value.red_mult != Sfixed8P8::ONE
    || value.green_mult != Sfixed8P8::ONE
    || value.blue_mult != Sfixed8P8::ONE
    || value.alpha_mult != Sfixed8P8::ONE;

  let mut to_write: Vec<i32> = Vec::new();
  if has_mult {
    to_write.extend_from_slice(&[
      value.red_mult.epsilons.into(),
      value.green_mult.epsilons.into(),
      value.blue_mult.epsilons.into(),
      value.alpha_mult.epsilons.into(),
    ]);
  }
  if has_add {
    to_write.extend_from_slice(&[
      value.red_add.into(),
      value.green_add.into(),
      value.blue_add.into(),
      value.alpha_add.into(),
    ]);
  }

  let bits = get_i32_min_bit_count(to_write.clone().into_iter());

  writer.write_bool_bits(has_add)?;
  writer.write_bool_bits(has_mult)?;
  writer.write_u32_bits(4, bits)?;

  for value in to_write {
    writer.write_i32_bits(bits, value)?;
  }

  Ok(())
}
