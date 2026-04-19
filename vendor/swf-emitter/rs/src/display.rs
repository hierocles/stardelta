use std::convert::TryInto;
use std::io;

use swf_types as ast;

use crate::basic_data_types::emit_straight_s_rgba8;
use crate::primitives::{emit_le_f32, emit_le_i16, emit_le_i32, emit_le_u16, emit_le_u32, emit_u8};

pub fn emit_blend_mode<W: io::Write>(writer: &mut W, value: ast::BlendMode) -> io::Result<()> {
  let code: u8 = match value {
    ast::BlendMode::Normal => 0,
    ast::BlendMode::Layer => 2,
    ast::BlendMode::Multiply => 3,
    ast::BlendMode::Screen => 4,
    ast::BlendMode::Lighten => 5,
    ast::BlendMode::Darken => 6,
    ast::BlendMode::Difference => 7,
    ast::BlendMode::Add => 8,
    ast::BlendMode::Subtract => 9,
    ast::BlendMode::Invert => 10,
    ast::BlendMode::Alpha => 11,
    ast::BlendMode::Erase => 12,
    ast::BlendMode::Overlay => 13,
    ast::BlendMode::Hardlight => 14,
  };
  emit_u8(writer, code)
}

pub fn emit_clip_actions_string<W: io::Write>(
  writer: &mut W,
  value: &[ast::ClipAction],
  extended_events: bool,
) -> io::Result<()> {
  emit_le_u16(writer, 0)?; // Reserved

  let mut event_union: ast::ClipEventFlags = ast::ClipEventFlags {
    load: false,
    enter_frame: false,
    unload: false,
    mouse_move: false,
    mouse_down: false,
    mouse_up: false,
    key_down: false,
    key_up: false,
    data: false,
    initialize: false,
    press: false,
    release: false,
    release_outside: false,
    roll_over: false,
    roll_out: false,
    drag_over: false,
    drag_out: false,
    key_press: false,
    construct: false,
  };

  for clip_action in value {
    event_union.load = event_union.load || clip_action.events.load;
    event_union.enter_frame = event_union.enter_frame || clip_action.events.enter_frame;
    event_union.unload = event_union.unload || clip_action.events.unload;
    event_union.mouse_move = event_union.mouse_move || clip_action.events.mouse_move;
    event_union.mouse_down = event_union.mouse_down || clip_action.events.mouse_down;
    event_union.mouse_up = event_union.mouse_up || clip_action.events.mouse_up;
    event_union.key_down = event_union.key_down || clip_action.events.key_down;
    event_union.key_up = event_union.key_up || clip_action.events.key_up;
    event_union.data = event_union.data || clip_action.events.data;
    event_union.initialize = event_union.initialize || clip_action.events.initialize;
    event_union.press = event_union.press || clip_action.events.press;
    event_union.release = event_union.release || clip_action.events.release;
    event_union.release_outside = event_union.release_outside || clip_action.events.release_outside;
    event_union.roll_over = event_union.roll_over || clip_action.events.roll_over;
    event_union.roll_out = event_union.roll_out || clip_action.events.roll_out;
    event_union.drag_over = event_union.drag_over || clip_action.events.drag_over;
    event_union.drag_out = event_union.drag_out || clip_action.events.drag_out;
    event_union.key_press = event_union.key_press || clip_action.events.key_press;
    event_union.construct = event_union.construct || clip_action.events.construct;
  }

  emit_clip_event_flags(writer, event_union, extended_events)?;
  for clip_action in value {
    emit_clip_actions(writer, clip_action, extended_events)?;
  }
  if extended_events {
    emit_le_u32(writer, 0)
  } else {
    emit_le_u16(writer, 0)
  }
}

pub fn emit_clip_event_flags<W: io::Write>(
  writer: &mut W,
  value: ast::ClipEventFlags,
  extended_events: bool,
) -> io::Result<()> {
  #[allow(clippy::identity_op)]
  let flags: u16 = 0
    | (if value.load { 1 << 0 } else { 0 })
    | (if value.enter_frame { 1 << 1 } else { 0 })
    | (if value.unload { 1 << 2 } else { 0 })
    | (if value.mouse_move { 1 << 3 } else { 0 })
    | (if value.mouse_down { 1 << 4 } else { 0 })
    | (if value.mouse_up { 1 << 5 } else { 0 })
    | (if value.key_down { 1 << 6 } else { 0 })
    | (if value.key_up { 1 << 7 } else { 0 })
    | (if value.data { 1 << 8 } else { 0 })
    | (if value.initialize { 1 << 9 } else { 0 })
    | (if value.press { 1 << 10 } else { 0 })
    | (if value.release { 1 << 11 } else { 0 })
    | (if value.release_outside { 1 << 12 } else { 0 })
    | (if value.roll_over { 1 << 13 } else { 0 })
    | (if value.roll_out { 1 << 14 } else { 0 })
    | (if value.drag_over { 1 << 15 } else { 0 });

  if !extended_events {
    return emit_le_u16(writer, flags);
  }

  let extended_flags: u32 = u32::from(flags)
    | (if value.drag_out { 1 << 16 } else { 0 })
    | (if value.key_press { 1 << 17 } else { 0 })
    | (if value.construct { 1 << 18 } else { 0 });

  emit_le_u32(writer, extended_flags)
}

pub fn emit_clip_actions<W: io::Write>(
  writer: &mut W,
  value: &ast::ClipAction,
  extended_events: bool,
) -> io::Result<()> {
  use std::io::Write;

  emit_clip_event_flags(writer, value.events, extended_events)?;

  let mut action_writer = Vec::new();
  if value.events.key_press {
    match value.key_code {
      Some(key_code) => emit_u8(&mut action_writer, key_code)?,
      None => panic!("Expected key_code to be defined"),
    }
  }
  action_writer.write_all(&value.actions)?;
  emit_le_u32(writer, action_writer.len().try_into().unwrap())?;
  writer.write_all(&action_writer)
}

pub fn emit_filter_list<W: io::Write>(writer: &mut W, value: &[ast::Filter]) -> io::Result<()> {
  emit_u8(writer, value.len().try_into().unwrap())?;
  for filter in value {
    emit_filter(writer, filter)?;
  }
  Ok(())
}

pub fn emit_filter<W: io::Write>(writer: &mut W, value: &ast::Filter) -> io::Result<()> {
  match value {
    ast::Filter::Bevel(filter) => {
      emit_u8(writer, 3)?;
      emit_bevel_filter(writer, filter)
    }
    ast::Filter::Blur(filter) => {
      emit_u8(writer, 1)?;
      emit_blur_filter(writer, filter)
    }
    ast::Filter::Convolution(filter) => {
      emit_u8(writer, 5)?;
      emit_convolution_filter(writer, filter)
    }
    ast::Filter::ColorMatrix(filter) => {
      emit_u8(writer, 6)?;
      emit_color_matrix_filter(writer, filter)
    }
    ast::Filter::DropShadow(filter) => {
      emit_u8(writer, 0)?;
      emit_drop_shadow_filter(writer, filter)
    }
    ast::Filter::Glow(filter) => {
      emit_u8(writer, 2)?;
      emit_glow_filter(writer, filter)
    }
    ast::Filter::GradientBevel(filter) => {
      emit_u8(writer, 7)?;
      emit_gradient_bevel_filter(writer, filter)
    }
    ast::Filter::GradientGlow(filter) => {
      emit_u8(writer, 4)?;
      emit_gradient_glow_filter(writer, filter)
    }
  }
}

pub fn emit_bevel_filter<W: io::Write>(writer: &mut W, value: &ast::filters::Bevel) -> io::Result<()> {
  assert!(value.passes < 0x10);

  emit_straight_s_rgba8(writer, value.shadow_color)?;
  emit_straight_s_rgba8(writer, value.highlight_color)?;
  emit_le_i32(writer, value.blur_x.epsilons)?;
  emit_le_i32(writer, value.blur_y.epsilons)?;
  emit_le_i32(writer, value.angle.epsilons)?;
  emit_le_i32(writer, value.distance.epsilons)?;
  emit_le_i16(writer, value.strength.epsilons)?;

  #[allow(clippy::identity_op)]
  let flags: u8 = 0
    | ((value.passes & 0x0f) << 0)
    | (if value.on_top { 1 << 4 } else { 0 })
    | (if value.composite_source { 1 << 5 } else { 0 })
    | (if value.knockout { 1 << 6 } else { 0 })
    | (if value.inner { 1 << 7 } else { 0 });
  emit_u8(writer, flags)
}

pub fn emit_blur_filter<W: io::Write>(writer: &mut W, value: &ast::filters::Blur) -> io::Result<()> {
  assert!(value.passes < 0x20);

  emit_le_i32(writer, value.blur_x.epsilons)?;
  emit_le_i32(writer, value.blur_y.epsilons)?;

  #[allow(clippy::identity_op)]
  let flags: u8 = 0
    // Skip bits [0, 2]
    | ((value.passes & 0x1f) << 3);
  emit_u8(writer, flags)
}

pub fn emit_color_matrix_filter<W: io::Write>(writer: &mut W, value: &ast::filters::ColorMatrix) -> io::Result<()> {
  assert_eq!(value.matrix.len(), 20);
  for coefficient in &value.matrix {
    emit_le_f32(writer, *coefficient)?;
  }
  Ok(())
}

pub fn emit_convolution_filter<W: io::Write>(writer: &mut W, value: &ast::filters::Convolution) -> io::Result<()> {
  assert!(value.matrix_width < 256);
  assert!(value.matrix_height < 256);
  assert_eq!(value.matrix.len(), value.matrix_width * value.matrix_height);

  emit_u8(writer, value.matrix_width.try_into().unwrap())?;
  emit_u8(writer, value.matrix_height.try_into().unwrap())?;
  emit_le_f32(writer, value.divisor)?;
  emit_le_f32(writer, value.bias)?;
  for coefficient in &value.matrix {
    emit_le_f32(writer, *coefficient)?;
  }
  emit_straight_s_rgba8(writer, value.default_color)?;

  #[allow(clippy::identity_op)]
  let flags: u8 = 0
    | (if value.preserve_alpha { 1 << 0 } else { 0 })
    | (if value.clamp { 1 << 1 } else { 0 });
  emit_u8(writer, flags)
}

pub fn emit_drop_shadow_filter<W: io::Write>(writer: &mut W, value: &ast::filters::DropShadow) -> io::Result<()> {
  assert!(value.passes < 0x20);

  emit_straight_s_rgba8(writer, value.color)?;
  emit_le_i32(writer, value.blur_x.epsilons)?;
  emit_le_i32(writer, value.blur_y.epsilons)?;
  emit_le_i32(writer, value.angle.epsilons)?;
  emit_le_i32(writer, value.distance.epsilons)?;
  emit_le_i16(writer, value.strength.epsilons)?;

  #[allow(clippy::identity_op)]
  let flags: u8 = 0
    | ((value.passes & 0x1f) << 0)
    | (if value.composite_source { 1 << 5 } else { 0 })
    | (if value.knockout { 1 << 6 } else { 0 })
    | (if value.inner { 1 << 7 } else { 0 });
  emit_u8(writer, flags)
}

pub fn emit_glow_filter<W: io::Write>(writer: &mut W, value: &ast::filters::Glow) -> io::Result<()> {
  assert!(value.passes < 0x20);

  emit_straight_s_rgba8(writer, value.color)?;
  emit_le_i32(writer, value.blur_x.epsilons)?;
  emit_le_i32(writer, value.blur_y.epsilons)?;
  emit_le_i16(writer, value.strength.epsilons)?;

  #[allow(clippy::identity_op)]
  let flags: u8 = 0
    | ((value.passes & 0x1f) << 0)
    | (if value.composite_source { 1 << 5 } else { 0 })
    | (if value.knockout { 1 << 6 } else { 0 })
    | (if value.inner { 1 << 7 } else { 0 });
  emit_u8(writer, flags)
}

pub fn emit_gradient_bevel_filter<W: io::Write>(writer: &mut W, value: &ast::filters::GradientBevel) -> io::Result<()> {
  assert!(value.passes < 0x10);
  assert!(value.gradient.len() < 256);

  emit_u8(writer, value.gradient.len().try_into().unwrap())?;
  for color_stop in &value.gradient {
    emit_straight_s_rgba8(writer, color_stop.color)?;
  }
  for color_stop in &value.gradient {
    emit_u8(writer, color_stop.ratio)?;
  }
  emit_le_i32(writer, value.blur_x.epsilons)?;
  emit_le_i32(writer, value.blur_y.epsilons)?;
  emit_le_i32(writer, value.angle.epsilons)?;
  emit_le_i32(writer, value.distance.epsilons)?;
  emit_le_i16(writer, value.strength.epsilons)?;

  #[allow(clippy::identity_op)]
  let flags: u8 = 0
    | ((value.passes & 0x0f) << 0)
    | (if value.on_top { 1 << 4 } else { 0 })
    | (if value.composite_source { 1 << 5 } else { 0 })
    | (if value.knockout { 1 << 6 } else { 0 })
    | (if value.inner { 1 << 7 } else { 0 });
  emit_u8(writer, flags)
}

pub fn emit_gradient_glow_filter<W: io::Write>(writer: &mut W, value: &ast::filters::GradientGlow) -> io::Result<()> {
  assert!(value.passes < 0x10);
  assert!(value.gradient.len() < 256);

  emit_u8(writer, value.gradient.len().try_into().unwrap())?;
  for color_stop in &value.gradient {
    emit_straight_s_rgba8(writer, color_stop.color)?;
  }
  for color_stop in &value.gradient {
    emit_u8(writer, color_stop.ratio)?;
  }
  emit_le_i32(writer, value.blur_x.epsilons)?;
  emit_le_i32(writer, value.blur_y.epsilons)?;
  emit_le_i32(writer, value.angle.epsilons)?;
  emit_le_i32(writer, value.distance.epsilons)?;
  emit_le_i16(writer, value.strength.epsilons)?;

  #[allow(clippy::identity_op)]
  let flags: u8 = 0
    | ((value.passes & 0x0f) << 0)
    | (if value.on_top { 1 << 4 } else { 0 })
    | (if value.composite_source { 1 << 5 } else { 0 })
    | (if value.knockout { 1 << 6 } else { 0 })
    | (if value.inner { 1 << 7 } else { 0 });
  emit_u8(writer, flags)
}
