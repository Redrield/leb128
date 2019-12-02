use super::{CONTINUATION_BIT, SIGN_BIT, low_bits_of_byte};
use std::fmt;
use std::io;
use bytes::buf::Buf;

/// Trait for reading signed and unsigned LEB128 encoded numbers
pub trait LEB128Read {
    /// Read a signed LEB128 number from the given `std::io::Read`able and
    /// return it or an error if reading failed.
    fn read_signed(&mut self) -> Result<(i64, usize), Error>;

    /// Read an unsigned LEB128 number from the given `std::io::Read`able and
    /// return it or an error if reading failed.
    fn read_unsigned(&mut self) -> Result<(u64, usize), Error>;
}

/// An enumeration of the possible errors that can occur when reading a
/// number encoded with LEB128.
#[derive(Debug)]
pub enum Error {
    /// There was an underlying IO error.
    IoError(io::Error),
    /// The number being read is larger than can be represented.
    Overflow,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IoError(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f,
               "leb128::read::Error: {}",
               ::std::error::Error::description(self))
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IoError(ref e) => e.description(),
            Error::Overflow => "The number being read is larger than can be represented",
        }
    }

    fn cause(&self) -> Option<&::std::error::Error> {
        match *self {
            Error::IoError(ref e) => Some(e),
            Error::Overflow => None,
        }
    }
}

impl<R> LEB128Read for R
    where R: Buf
{
    fn read_signed(&mut self) -> Result<(i64, usize), Error> {
        let mut result = 0;
        let mut shift = 0;
        let size = 64;
        let mut byte;
        let mut bytes_read = 0;

        loop {
            if !self.has_remaining() {
                return Err(Error::IoError(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough data")))
            }

            byte = self.get_u8();
            bytes_read += 1;
            if shift == 63 && byte != 0x00 && byte != 0x7f {
                return Err(Error::Overflow);
            }

            let low_bits = low_bits_of_byte(byte) as i64;
            result |= low_bits << shift;
            shift += 7;

            if byte & CONTINUATION_BIT == 0 {
                break;
            }
        }

        if shift < size && (SIGN_BIT & byte) == SIGN_BIT {
            // Sign extend the result.
            result |= !0 << shift;
        }

        Ok((result, bytes_read))
    }

    fn read_unsigned(&mut self) -> Result<(u64, usize), Error> {
        let mut result = 0;
        let mut shift = 0;
        let mut bytes_read = 0;

        loop {
            if !self.has_remaining() {
                return Err(Error::IoError(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough data")))
            }

            let byte = self.get_u8();
            bytes_read += 1;

            if shift == 63 && byte != 0x00 && byte != 0x01 {
                return Err(Error::Overflow);
            }

            let low_bits = low_bits_of_byte(byte) as u64;
            result |= low_bits << shift;

            if byte & CONTINUATION_BIT == 0 {
                return Ok((result, bytes_read));
            }

            shift += 7;
        }
    }
}