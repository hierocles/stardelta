use std::convert::TryFrom;
use std::io;

use swf_types as ast;

use crate::basic_data_types::{emit_s_rgb8, emit_straight_s_rgba8};
use crate::primitives::emit_u8;

fn gradient_spread_to_code(value: ast::GradientSpread) -> u8 {
  match value {
    ast::GradientSpread::Pad => 0,
    ast::GradientSpread::Reflect => 1,
    ast::GradientSpread::Repeat => 2,
  }
}

fn color_space_to_code(value: ast::ColorSpace) -> u8 {
  match value {
    ast::ColorSpace::LinearRgb => 1,
    ast::ColorSpace::SRgb => 0,
  }
}

pub(crate) fn emit_gradient<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &ast::Gradient,
  with_alpha: bool,
) -> io::Result<()> {
  assert!(value.colors.len() <= 0x0f);
  #[allow(clippy::identity_op)]
  let flags: u8 = 0
    | ((u8::try_from(value.colors.len()).unwrap() & 0x0f) << 0)
    | ((gradient_spread_to_code(value.spread) & 0b11) << 4)
    | ((color_space_to_code(value.color_space) & 0b11) << 6);
  emit_u8(writer, flags)?;

  for color_stop in &value.colors {
    emit_color_stop(writer, color_stop, with_alpha)?;
  }

  Ok(())
}

pub(crate) fn emit_color_stop<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &ast::ColorStop,
  with_alpha: bool,
) -> io::Result<()> {
  emit_u8(writer, value.ratio)?;
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

pub(crate) fn emit_morph_gradient<W: io::Write + ?Sized>(writer: &mut W, value: &ast::MorphGradient) -> io::Result<()> {
  assert!(value.colors.len() <= 0x0f);
  #[allow(clippy::identity_op)]
  let flags: u8 = 0
    | ((u8::try_from(value.colors.len()).unwrap() & 0x0f) << 0)
    | ((gradient_spread_to_code(value.spread) & 0b11) << 4)
    | ((color_space_to_code(value.color_space) & 0b11) << 6);
  emit_u8(writer, flags)?;

  for color_stop in &value.colors {
    emit_morph_color_stop(writer, color_stop)?;
  }

  Ok(())
}

pub(crate) fn emit_morph_color_stop<W: io::Write + ?Sized>(
  writer: &mut W,
  value: &ast::MorphColorStop,
) -> io::Result<()> {
  emit_color_stop(
    writer,
    &ast::ColorStop {
      ratio: value.ratio,
      color: value.color,
    },
    true,
  )?;
  emit_color_stop(
    writer,
    &ast::ColorStop {
      ratio: value.morph_ratio,
      color: value.morph_color,
    },
    true,
  )
}
