use std::convert::TryInto;
use std::io;

use crate::basic_data_types::emit_rect;
use crate::error::SwfEmitError;
use crate::primitives::{emit_le_u16, emit_le_u32, emit_u8};
use crate::tags::emit_tag_string;
use ast::CompressionMethod;
use swf_types as ast;

const SWF_SIGNATURE_SIZE: usize = 8;

pub fn emit_swf<W: io::Write>(
  writer: &mut W,
  value: &ast::Movie,
  compression_method: ast::CompressionMethod,
) -> Result<(), SwfEmitError> {
  let write_movie_fn = match compression_method {
    ast::CompressionMethod::None => W::write_all,
    #[cfg(feature="deflate")]
    ast::CompressionMethod::Deflate => write_bytes_deflate,
    #[cfg(feature="lzma")]
    ast::CompressionMethod::Lzma => write_bytes_lzma,
    #[allow(unreachable_patterns)]
    method => return Err(SwfEmitError::UnsupportedCompression(method)),
  };

  let mut movie_bytes = Vec::new();
  let signature = emit_movie_bytes_to_buf(&mut movie_bytes, value, compression_method).map_err(SwfEmitError::Io)?;
  emit_swf_signature(writer, &signature).map_err(SwfEmitError::Io)?;
  write_movie_fn(writer, &movie_bytes).map_err(SwfEmitError::Io)
}

/// Special case of [`emit_swf`] that avoids an extra copy
pub(crate) fn emit_uncompressed_swf_to_buf(buf: &mut Vec<u8>, value: &ast::Movie) -> Result<(), SwfEmitError> {
  let start = buf.len();
  buf.extend([0; SWF_SIGNATURE_SIZE]);
  let signature = emit_movie_bytes_to_buf(buf, value, CompressionMethod::None).map_err(SwfEmitError::Io)?;

  let mut header = &mut buf[start..(start + SWF_SIGNATURE_SIZE)];
  emit_swf_signature(&mut header, &signature).map_err(SwfEmitError::Io)
}

fn emit_movie_bytes_to_buf(buf: &mut Vec<u8>, value: &ast::Movie, compression_method: CompressionMethod) -> io::Result<ast::SwfSignature> {
  let before = buf.len();
  emit_movie(buf, value)?;
  let uncompressed_file_length = SWF_SIGNATURE_SIZE + (buf.len() - before);
  Ok(ast::SwfSignature {
    compression_method,
    swf_version: value.header.swf_version,
    uncompressed_file_length,
  })
}

#[cfg(feature="lzma")]
fn write_bytes_lzma<W: io::Write>(writer: &mut W, mut bytes: &[u8]) -> io::Result<()> {
  lzma_rs::lzma_compress(&mut bytes, writer)
}

#[cfg(feature="deflate")]
fn write_bytes_deflate<W: io::Write>(writer: &mut W, bytes: &[u8]) -> io::Result<()> {
  use miniz_oxide::deflate::{core, CompressionLevel};

  let mut compressor = core::CompressorOxide::default();
  compressor.set_format_and_level(miniz_oxide::DataFormat::Zlib, CompressionLevel::DefaultLevel as u8);

  let mut result = Ok(());
  let (status, written) = core::compress_to_output(&mut compressor, bytes, core::TDEFLFlush::Finish, |out| {
    result = writer.write_all(out);
    result.is_ok()
  });
  
  // Check the compression status; panicking here is a bug.
  match status {
    core::TDEFLStatus::PutBufFailed => assert!(result.is_err(), "miniz_oxide: unexpected error"),
    core::TDEFLStatus::BadParam => panic!("miniz_oxide: bad deflate params"),
    core::TDEFLStatus::Okay => panic!("miniz_oxide: unexpected partial compression"),
    core::TDEFLStatus::Done => assert!(result.is_ok() && written == bytes.len(), "miniz_oxide: unexpected success"),
  }
  result
}

pub fn emit_swf_signature<W: io::Write>(writer: &mut W, value: &ast::SwfSignature) -> io::Result<()> {
  assert!(value.uncompressed_file_length <= 0xffffffff);

  emit_compression_method(writer, value.compression_method)?;
  emit_u8(writer, value.swf_version)?;
  emit_le_u32(writer, value.uncompressed_file_length.try_into().unwrap())
}

pub fn emit_compression_method<W: io::Write>(writer: &mut W, value: ast::CompressionMethod) -> io::Result<()> {
  let code: &'static [u8] = match value {
    ast::CompressionMethod::Deflate => b"CWS",
    ast::CompressionMethod::Lzma => b"ZWS",
    ast::CompressionMethod::None => b"FWS",
  };
  writer.write_all(code)
}

pub fn emit_movie<W: io::Write>(writer: &mut W, value: &ast::Movie) -> io::Result<()> {
  emit_header(writer, &value.header)?;
  emit_tag_string(writer, &value.tags, value.header.swf_version)
}

pub fn emit_header<W: io::Write>(writer: &mut W, value: &ast::Header) -> io::Result<()> {
  emit_rect(writer, &value.frame_size)?;
  emit_le_u16(writer, value.frame_rate.epsilons)?;
  emit_le_u16(writer, value.frame_count)
}
