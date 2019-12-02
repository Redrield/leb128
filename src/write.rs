use super::{CONTINUATION_BIT, low_bits_of_u64};
use std::io;
use bytes::{BytesMut, BufMut};

/// Trait for writing signed and unsigned LEB128 encoded numbers
pub trait LEB128Write {
    /// Write the given signed number using the LEB128 encoding to the given
    /// `std::io::Write`able. Returns the number of bytes written to `w`, or an
    /// error if writing failed.
    fn write_signed(&mut self, val: i64) -> Result<usize, io::Error>;

    /// Write the given unsigned number using the LEB128 encoding to the given
    /// `std::io::Write`able. Returns the number of bytes written to `w`, or an
    /// error if writing failed.
    fn write_unsigned(&mut self, val: u64) -> Result<usize, io::Error>;
}

impl<W> LEB128Write for W
    where W: BufMut
{
    fn write_signed(&mut self, mut val: i64) -> Result<usize, io::Error> {
        let mut bytes_written = 0;
        loop {
            let mut byte = val as u8;
            // Keep the sign bit for testing
            val >>= 6;
            let done = val == 0 || val == -1;
            if done {
                byte &= !CONTINUATION_BIT;
            } else {
                // Remove the sign bit
                val >>= 1;
                // More bytes to come, so set the continuation bit.
                byte |= CONTINUATION_BIT;
            }

            self.put_u8(byte);
            bytes_written += 1;

            if done {
                return Ok(bytes_written);
            }
        }
    }

    fn write_unsigned(&mut self, mut val: u64) -> Result<usize, io::Error> {
        let mut bytes_written = 0;
        loop {
            let mut byte = low_bits_of_u64(val);
            val >>= 7;
            if val != 0 {
                // More bytes to come, so set the continuation bit.
                byte |= CONTINUATION_BIT;
            }

            let buf = [byte];
            self.put_u8(byte);
            bytes_written += 1;

            if val == 0 {
                return Ok(bytes_written);
            }
        }
    }
}