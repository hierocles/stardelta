pub mod basic_data_types;
pub mod bit_count;
pub mod button;
pub mod display;
mod error;
pub mod gradient;
pub mod io_bits;
pub mod morph_shape;
pub mod movie;
pub mod primitives;
pub mod shape;
pub mod sound;
pub mod tags;
pub mod text;

use crate::movie::emit_swf as write_swf;
use crate::tags::emit_tag as write_tag;
use swf_types::{CompressionMethod, Movie, Tag};

pub use error::SwfEmitError;

pub fn emit_swf(value: &Movie, compression_method: CompressionMethod) -> Result<Vec<u8>, SwfEmitError> {
  let mut swf_writer: Vec<u8> = Vec::new();
  match compression_method {
    CompressionMethod::None => crate::movie::emit_uncompressed_swf_to_buf(&mut swf_writer, value)?,
    _ => write_swf(&mut swf_writer, value, compression_method)?,
  }
  Ok(swf_writer)
}

pub fn emit_tag(value: &Tag, swf_version: u8) -> std::io::Result<Vec<u8>> {
  let mut tag_writer: Vec<u8> = Vec::new();
  write_tag(&mut tag_writer, value, swf_version)?;
  Ok(tag_writer)
}

#[cfg(test)]
mod tests {
  use std::path::Path;

  use ::test_generator::test_expand_paths;
  use swf_types::Matrix;
  use swf_types::Rect;
  use swf_types::{ColorTransformWithAlpha, CompressionMethod, Header, Movie, SwfSignature, Tag};

  use crate::basic_data_types::{emit_color_transform_with_alpha, emit_leb128_u32, emit_matrix, emit_rect};
  use crate::movie::{emit_header, emit_swf_signature};
  use crate::primitives::emit_le_f16;
  use crate::{emit_swf, emit_tag};

  test_expand_paths! { test_emit_movie; "../tests/movies/*/" }
  fn test_emit_movie(path: &str) {
    let path: &Path = Path::new(path);
    let _name = path
      .components()
      .last()
      .unwrap()
      .as_os_str()
      .to_str()
      .expect("Failed to retrieve sample name");

    let ast_path = path.join("ast.json");
    let ast_file = ::std::fs::File::open(ast_path).expect("Failed to open AST file");
    let ast_reader = ::std::io::BufReader::new(ast_file);
    let value: Movie = serde_json::from_reader(ast_reader).expect("Failed to read value");

    const COMPRESSION_METHODS: &[(CompressionMethod, &str)] = &[
      (CompressionMethod::None, "local-main.rs.swf"),
      #[cfg(feature="deflate")]
      (CompressionMethod::Deflate, "local-main-deflate.rs.swf"),
      #[cfg(feature="lzma")]
      (CompressionMethod::Lzma, "local-main-lzma.rs.swf"),
    ];

    for (method, filename) in COMPRESSION_METHODS {
      let actual_bytes = emit_swf(&value, *method).unwrap();

      let actual_movie_path = path.join(*filename);
      ::std::fs::write(actual_movie_path, &actual_bytes).expect("Failed to write actual SWF");
  
      let actual_movie = swf_parser::parse_swf(&actual_bytes).expect("Failed to parse movie");

      assert_eq!(actual_movie, value);
    }
  }

  test_expand_paths! { test_emit_tag; "../tests/tags/*/*/" }
  fn test_emit_tag(path: &str) {
    let path: &Path = Path::new(path);
    let name = path
      .components()
      .last()
      .unwrap()
      .as_os_str()
      .to_str()
      .expect("Failed to retrieve sample name");

    let value_path = path.join("value.json");
    let value_file = ::std::fs::File::open(value_path).expect("Failed to open value file");
    let value_reader = ::std::io::BufReader::new(value_file);
    let value = serde_json::from_reader::<_, Tag>(value_reader).expect("Failed to read value");

    let swf_version: u8 = match name {
      "po2-swf5" => 5,
      _ => 10,
    };

    let actual_bytes = emit_tag(&value, swf_version).unwrap();

    let expected_bytes: Vec<u8> = {
      match ::std::fs::read(path.join("output.bytes")) {
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
          ::std::fs::read(path.join("input.bytes")).expect("Failed to read expected output (input.bytes)")
        }
        r => r.expect("Failed to read expected output (output.bytes)"),
      }
    };

    assert_eq!(expected_bytes, actual_bytes);
  }

  macro_rules! test_various_ref_emitter_impl {
    ($name:ident, $glob:expr, $emitter:ident, $type:ty) => {
      test_expand_paths! { $name; $glob }
      fn $name(path: &str) {
        let path: &Path = Path::new(path);
        let _name = path
          .components()
          .last()
          .unwrap()
          .as_os_str()
          .to_str()
          .expect("Failed to retrieve sample name");

        let value_path = path.join("value.json");
        let value_file = ::std::fs::File::open(value_path).expect("Failed to open value file");
        let value_reader = ::std::io::BufReader::new(value_file);
        let value = serde_json::from_reader::<_, $type>(value_reader).expect("Failed to read value");

        let mut actual_bytes = Vec::new();
        $emitter(&mut actual_bytes, &value).expect("Failed to emit");

        let expected_bytes: Vec<u8> = {
          match ::std::fs::read(path.join("output.bytes")) {
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
              ::std::fs::read(path.join("input.bytes")).expect("Failed to read expected output (input.bytes)")
            }
            r => r.expect("Failed to read expected output (output.bytes)"),
          }
        };

        assert_eq!(expected_bytes, actual_bytes);
      }
    };
  }

  // TODO: Avoid duplication with `test_various_ref_emitter_impl`. The only difference is that the value is passed by
  //       copy instead of reference.
  macro_rules! test_various_copy_emitter_impl {
    ($name:ident, $glob:expr, $emitter:ident, $type:ty) => {
      test_expand_paths! { $name; $glob }
      fn $name(path: &str) {
        let path: &Path = Path::new(path);
        let _name = path
          .components()
          .last()
          .unwrap()
          .as_os_str()
          .to_str()
          .expect("Failed to retrieve sample name");

        let value_path = path.join("value.json");
        let value_file = ::std::fs::File::open(value_path).expect("Failed to open value file");
        let value_reader = ::std::io::BufReader::new(value_file);
        let value = serde_json::from_reader::<_, $type>(value_reader).expect("Failed to read value");

        let mut actual_bytes = Vec::new();
        $emitter(&mut actual_bytes, value).expect("Failed to emit");

        let expected_bytes: Vec<u8> = {
          match ::std::fs::read(path.join("output.bytes")) {
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
              ::std::fs::read(path.join("input.bytes")).expect("Failed to read expected output (input.bytes)")
            }
            r => r.expect("Failed to read expected output (output.bytes)"),
          }
        };

        assert_eq!(expected_bytes, actual_bytes);
      }
    };
  }

  test_various_ref_emitter_impl!(
    test_emit_color_transform_with_alpha,
    "../tests/various/color-transform-with-alpha/*/",
    emit_color_transform_with_alpha,
    ColorTransformWithAlpha
  );

  test_various_copy_emitter_impl!(test_emit_le_f16, "../tests/various/float16-le/*/", emit_le_f16, f32);

  test_various_ref_emitter_impl!(test_emit_header, "../tests/various/header/*/", emit_header, Header);

  test_various_ref_emitter_impl!(test_emit_matrix, "../tests/various/matrix/*/", emit_matrix, Matrix);

  test_various_ref_emitter_impl!(test_emit_rect, "../tests/various/rect/*/", emit_rect, Rect);

  test_various_ref_emitter_impl!(
    test_emit_swf_signature,
    "../tests/various/swf-signature/*/",
    emit_swf_signature,
    SwfSignature
  );

  test_various_copy_emitter_impl!(
    test_emit_leb128_u32,
    "../tests/various/uint32-leb128/*/",
    emit_leb128_u32,
    u32
  );
}
