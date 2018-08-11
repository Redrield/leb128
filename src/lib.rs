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
#[cfg(feature = "use-bytes")]
extern crate bytes;

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

#[cfg(all(test, feature = "use-bytes"))]
mod tests_bytes;

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use std;
    use std::io;

    #[test]
    fn test_low_bits_of_byte() {
        for i in 0..127 {
            assert_eq!(i, low_bits_of_byte(i));
            assert_eq!(i, low_bits_of_byte(i | CONTINUATION_BIT));
        }
    }

    #[test]
    fn test_low_bits_of_u64() {
        for i in 0u64..127 {
            assert_eq!(i as u8, low_bits_of_u64(1 << 16 | i));
            assert_eq!(i as u8,
                       low_bits_of_u64(i << 16 | i | (CONTINUATION_BIT as u64)));
        }
    }

    use read::LEB128Read;
    use write::LEB128Write;

    // Examples from the DWARF 4 standard, section 7.6, figure 22.
    #[test]
    fn test_read_unsigned() {
        let buf = [2u8];
        let mut readable = &buf[..];
        assert_eq!(2,
                   readable.read_unsigned().expect("Should read number"));

        let buf = [127u8];
        let mut readable = &buf[..];
        assert_eq!(127,
                   readable.read_unsigned().expect("Should read number"));

        let buf = [CONTINUATION_BIT, 1];
        let mut readable = &buf[..];
        assert_eq!(128,
                   readable.read_unsigned().expect("Should read number"));

        let buf = [1u8 | CONTINUATION_BIT, 1];
        let mut readable = &buf[..];
        assert_eq!(129,
                   readable.read_unsigned().expect("Should read number"));

        let buf = [2u8 | CONTINUATION_BIT, 1];
        let mut readable = &buf[..];
        assert_eq!(130,
                   readable.read_unsigned().expect("Should read number"));

        let buf = [57u8 | CONTINUATION_BIT, 100];
        let mut readable = &buf[..];
        assert_eq!(12857,
                   readable.read_unsigned().expect("Should read number"));
    }

    // Examples from the DWARF 4 standard, section 7.6, figure 23.
    #[test]
    fn test_read_signed() {
        let buf = [2u8];
        let mut readable = &buf[..];
        assert_eq!(2, readable.read_signed().expect("Should read number"));

        let buf = [0x7eu8];
        let mut readable = &buf[..];
        assert_eq!(-2, readable.read_signed().expect("Should read number"));

        let buf = [127u8 | CONTINUATION_BIT, 0];
        let mut readable = &buf[..];
        assert_eq!(127,
                   readable.read_signed().expect("Should read number"));

        let buf = [1u8 | CONTINUATION_BIT, 0x7f];
        let mut readable = &buf[..];
        assert_eq!(-127,
                   readable.read_signed().expect("Should read number"));

        let buf = [CONTINUATION_BIT, 1];
        let mut readable = &buf[..];
        assert_eq!(128,
                   readable.read_signed().expect("Should read number"));

        let buf = [CONTINUATION_BIT, 0x7f];
        let mut readable = &buf[..];
        assert_eq!(-128,
                   readable.read_signed().expect("Should read number"));

        let buf = [1u8 | CONTINUATION_BIT, 1];
        let mut readable = &buf[..];
        assert_eq!(129,
                   readable.read_signed().expect("Should read number"));

        let buf = [0x7fu8 | CONTINUATION_BIT, 0x7e];
        let mut readable = &buf[..];
        assert_eq!(-129,
                   readable.read_signed().expect("Should read number"));
    }

    #[test]
    fn test_read_signed_63_bits() {
        let buf = [CONTINUATION_BIT,
            CONTINUATION_BIT,
            CONTINUATION_BIT,
            CONTINUATION_BIT,
            CONTINUATION_BIT,
            CONTINUATION_BIT,
            CONTINUATION_BIT,
            CONTINUATION_BIT,
            0x40];
        let mut readable = &buf[..];
        assert_eq!(-0x4000000000000000,
                   readable.read_signed().expect("Should read number"));
    }

    #[test]
    fn test_read_unsigned_not_enough_data() {
        let buf = [CONTINUATION_BIT];
        let mut readable = &buf[..];
        match readable.read_unsigned() {
            Err(read::Error::IoError(e)) => assert_eq!(e.kind(), io::ErrorKind::UnexpectedEof),
            otherwise => panic!("Unexpected: {:?}", otherwise),
        }
    }

    #[test]
    fn test_read_signed_not_enough_data() {
        let buf = [CONTINUATION_BIT];
        let mut readable = &buf[..];
        match readable.read_signed() {
            Err(read::Error::IoError(e)) => assert_eq!(e.kind(), io::ErrorKind::UnexpectedEof),
            otherwise => panic!("Unexpected: {:?}", otherwise),
        }
    }

    #[test]
    fn test_write_unsigned_not_enough_space() {
        let mut buf = [0; 1];
        let mut writable = &mut buf[..];
        match writable.write_unsigned(128) {
            Err(e) => assert_eq!(e.kind(), io::ErrorKind::WriteZero),
            otherwise => panic!("Unexpected: {:?}", otherwise),
        }
    }

    #[test]
    fn test_write_signed_not_enough_space() {
        let mut buf = [0; 1];
        let mut writable = &mut buf[..];
        match writable.write_signed(128) {
            Err(e) => assert_eq!(e.kind(), io::ErrorKind::WriteZero),
            otherwise => panic!("Unexpected: {:?}", otherwise),
        }
    }

    #[test]
    fn dogfood_signed() {
        fn inner(i: i64) {
            let mut buf = [0u8; 1024];

            {
                let mut writable = &mut buf[..];
                writable.write_signed(i).expect("Should write signed number");
            }

            let mut readable = &buf[..];
            let result = readable.read_signed().expect("Should be able to read it back again");
            assert_eq!(i, result);
        }
        for i in -513..513 {
            inner(i);
        }
        inner(std::i64::MIN);
    }

    #[test]
    fn dogfood_unsigned() {
        for i in 0..1025 {
            let mut buf = [0u8; 1024];

            {
                let mut writable = &mut buf[..];
                writable.write_unsigned(i).expect("Should write signed number");
            }

            let mut readable = &buf[..];
            let result = readable.read_unsigned()
                .expect("Should be able to read it back again");
            assert_eq!(i, result);
        }
    }

    #[test]
    fn test_read_unsigned_overflow() {
        let buf = [2u8 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            1];
        let mut readable = &buf[..];
        assert!(readable.read_unsigned().is_err());
    }

    #[test]
    fn test_read_signed_overflow() {
        let buf = [2u8 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            2 | CONTINUATION_BIT,
            1];
        let mut readable = &buf[..];
        assert!(readable.read_signed().is_err());
    }

    #[test]
    fn test_read_multiple() {
        let buf = [2u8 | CONTINUATION_BIT, 1u8, 1u8];

        let mut readable = &buf[..];
        assert_eq!(readable.read_unsigned().expect("Should read first number"),
                   130u64);
        assert_eq!(readable.read_unsigned().expect("Should read first number"),
                   1u64);
    }
}
