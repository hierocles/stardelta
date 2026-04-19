use std::cmp::min;
use std::io;

pub trait WriteBits {
  fn align(&mut self) -> io::Result<()>;

  fn write_bool_bits(&mut self, value: bool) -> io::Result<()>;

  fn write_i32_bits(&mut self, bits: u32, value: i32) -> io::Result<()>;

  fn write_u32_bits(&mut self, bits: u32, value: u32) -> io::Result<()>;

  /// Align the writer and return a byte writer
  fn write_bytes(&mut self) -> io::Result<&mut dyn io::Write>;
}

pub struct BitsWriter<W: io::Write> {
  bit: u32,
  buffer: u8,
  inner: W,
}

impl<W: io::Write> BitsWriter<W> {
  pub fn new(inner: W) -> BitsWriter<W> {
    BitsWriter {
      bit: 0,
      buffer: 0,
      inner,
    }
  }

  pub fn into_inner(mut self) -> io::Result<W> {
    self.align()?;
    Ok(self.inner)
  }
}

impl<W: io::Write> WriteBits for BitsWriter<W> {
  fn align(&mut self) -> io::Result<()> {
    if self.bit != 0 {
      self.inner.write_all(&[self.buffer])?;
      self.bit = 0;
      self.buffer = 0;
    }
    Ok(())
  }

  fn write_bool_bits(&mut self, value: bool) -> io::Result<()> {
    debug_assert!(self.bit < 8);

    if value {
      self.buffer |= 1 << (7 - self.bit);
    }
    self.bit += 1;

    if self.bit == 8 {
      self.inner.write_all(&[self.buffer])?;
      self.bit = 0;
      self.buffer = 0;
    }

    Ok(())
  }

  fn write_i32_bits(&mut self, bits: u32, value: i32) -> io::Result<()> {
    // TODO: Add debug assertions to check the range of `value`
    if value < 0 {
      self.write_u32_bits(bits, ((1 << bits) + value) as u32)
    } else {
      self.write_u32_bits(bits, value as u32)
    }
  }

  fn write_u32_bits(&mut self, mut bits: u32, value: u32) -> io::Result<()> {
    // TODO: Add debug assertions to check the range of `value`
    debug_assert!(bits <= 32);
    debug_assert!(self.bit < 8);

    while bits > 0 {
      let available_bits = 8 - self.bit;
      let consumed_bits = min(available_bits, bits);
      debug_assert!((1..=8).contains(&consumed_bits));

      let chunk: u8 = ((value >> (bits - consumed_bits)) & ((1 << consumed_bits) - 1)) as u8;
      self.buffer |= chunk << (available_bits - consumed_bits);
      bits -= consumed_bits;
      self.bit += consumed_bits;

      if self.bit == 8 {
        self.inner.write_all(&[self.buffer])?;
        self.bit = 0;
        self.buffer = 0;
      }
    }

    Ok(())
  }

  fn write_bytes(&mut self) -> io::Result<&mut dyn io::Write> {
    self.align()?;
    Ok(&mut self.inner)
  }
}
