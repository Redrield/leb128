//! Read and write DWARF's "Little Endian Base 128" (LEB128) variable length
//! integer encoding.
//!
//! The implementation is a direct translation of the psuedocode in the DWARF 4
//! standard's appendix C.
//!
//! Read and write signed integers:
//!
//! ```
//! use leb128::write::LEB128Write;
//! use leb128::read::LEB128Read;
//!
//! let mut buf = [0; 1024];
//!
//! // Write to anything that implements `std::io::Write`.
//! {
//!     let mut writable = &mut buf[..];
//!     writable.write_signed(-12345).expect("Should write number");
//! }
//!
//! // Read from anything that implements `std::io::Read`.
//! let mut readable = &buf[..];
//! let val = readable.read_signed().expect("Should read number");
//! assert_eq!(val, -12345);
//! ```
//!
//! Or read and write unsigned integers:
//!
//! ```
//! use leb128::write::LEB128Write;
//! use leb128::read::LEB128Read;
//!
//! let mut buf = [0; 1024];
//!
//! {
//!     let mut writable = &mut buf[..];
//!     writable.write_unsigned(98765).expect("Should write number");
//! }
//!
//! let mut readable = &buf[..];
//! let val = readable.read_signed().expect("Should read number");
//! assert_eq!(val, 98765);
//! ```

#![deny(missing_docs)]

#[doc(hidden)]
pub const CONTINUATION_BIT: u8 = 1 << 7;
#[doc(hidden)]
pub const SIGN_BIT: u8 = 1 << 6;

#[doc(hidden)]
#[inline]
pub fn low_bits_of_byte(byte: u8) -> u8 {
    byte & !CONTINUATION_BIT
}

#[doc(hidden)]
#[inline]
pub fn low_bits_of_u64(val: u64) -> u8 {
    let byte = val & (std::u8::MAX as u64);
    low_bits_of_byte(byte as u8)
}

/// A module for reading signed and unsigned integers that have been LEB128
/// encoded.
pub mod read;

/// A module for writing integers encoded as LEB128.
pub mod write;

pub use self::read::LEB128Read;
pub use self::write::LEB128Write;

#[cfg(test)]
mod tests_bytes;

