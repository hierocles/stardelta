use std::fmt;
use std::error::Error;
use std::io;

use swf_types::CompressionMethod;

#[derive(Debug)]
pub enum SwfEmitError {
    Io(io::Error),
    UnsupportedCompression(CompressionMethod),
}

impl Error for SwfEmitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for SwfEmitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => fmt::Display::fmt(err, f),
            Self::UnsupportedCompression(method) => write!(f, "Unsupported compression method: {:?}", method),
        }
    }
}
