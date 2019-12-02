use super::*;
use std;
use std::io;
use bytes::{BytesMut, Bytes, BufMut, Buf};

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
    let mut readable = Bytes::from(&[2u8][..]);
    assert_eq!(2,
               readable.read_unsigned().expect("Should read number").0);

    let mut readable = Bytes::from(&[127u8][..]);
    assert_eq!(127,
               readable.read_unsigned().expect("Should read number").0);

    let mut readable = Bytes::from(&[CONTINUATION_BIT, 1][..]);
    assert_eq!(128,
               readable.read_unsigned().expect("Should read number").0);

    let mut readable = Bytes::from(&[1u8 | CONTINUATION_BIT, 1][..]);
    assert_eq!(129,
               readable.read_unsigned().expect("Should read number").0);

    let mut readable = Bytes::from(&[2u8 | CONTINUATION_BIT, 1][..]);
    assert_eq!(130,
               readable.read_unsigned().expect("Should read number").0);

    let mut readable = Bytes::from(&[57u8 | CONTINUATION_BIT, 100][..]);
    assert_eq!(12857,
               readable.read_unsigned().expect("Should read number").0);
}

// Examples from the DWARF 4 standard, section 7.6, figure 23.
#[test]
fn test_read_signed() {
    let mut readable = Bytes::from(&[2u8][..]);
    assert_eq!(2, readable.read_signed().expect("Should read number").0);

    let mut readable = Bytes::from(&[0x7eu8][..]);
    assert_eq!(-2, readable.read_signed().expect("Should read number").0);

    let mut readable = Bytes::from(&[127u8 | CONTINUATION_BIT, 0][..]);
    assert_eq!(127, readable.read_signed().expect("Should read number").0);

    let mut readable = Bytes::from(&[1u8 | CONTINUATION_BIT, 0x7f][..]);
    assert_eq!(-127, readable.read_signed().expect("Should read number").0);

    let mut readable = Bytes::from(&[CONTINUATION_BIT, 1][..]);
    assert_eq!(128, readable.read_signed().expect("Should read number").0);

    let mut readable = Bytes::from(&[CONTINUATION_BIT, 0x7f][..]);
    assert_eq!(-128, readable.read_signed().expect("Should read number").0);

    let mut readable = Bytes::from(&[1u8 | CONTINUATION_BIT, 1][..]);
    assert_eq!(129, readable.read_signed().expect("Should read number").0);

    let mut readable = Bytes::from(&[0x7fu8 | CONTINUATION_BIT, 0x7e][..]);
    assert_eq!(-129, readable.read_signed().expect("Should read number").0);
}

#[test]
fn test_read_signed_63_bits() {
    let mut readable = Bytes::from(&[CONTINUATION_BIT,
        CONTINUATION_BIT,
        CONTINUATION_BIT,
        CONTINUATION_BIT,
        CONTINUATION_BIT,
        CONTINUATION_BIT,
        CONTINUATION_BIT,
        CONTINUATION_BIT,
        0x40][..]);
    assert_eq!(-0x4000000000000000,
               readable.read_signed().expect("Should read number").0);
}

#[test]
fn test_read_unsigned_not_enough_data() {
    let mut readable = Bytes::from(&[CONTINUATION_BIT][..]);
    match readable.read_unsigned() {
        Err(read::Error::IoError(e)) => assert_eq!(e.kind(), io::ErrorKind::UnexpectedEof),
        otherwise => panic!("Unexpected: {:?}", otherwise),
    }
}

#[test]
fn test_read_signed_not_enough_data() {
    let mut readable = Bytes::from(&[CONTINUATION_BIT][..]);
    match readable.read_signed() {
        Err(read::Error::IoError(e)) => assert_eq!(e.kind(), io::ErrorKind::UnexpectedEof),
        otherwise => panic!("Unexpected: {:?}", otherwise),
    }
}

#[test]
fn dogfood_signed() {
    fn inner(i: i64) {
        let mut writable = BytesMut::new();

        {
            writable.write_signed(i).expect("Should write signed number");
        }

        let mut readable = writable.freeze();
        let result = readable.read_signed().expect("Should be able to read it back again");
        assert_eq!(i, result.0);
    }
    for i in -513..513 {
        inner(i);
    }
    inner(std::i64::MIN);
}

#[test]
fn dogfood_unsigned() {
    for i in 0..1025 {
        let mut writable = BytesMut::new();

        {
            writable.write_unsigned(i).expect("Should write signed number");
        }

        let mut readable = writable.freeze();
        let result = readable.read_unsigned()
            .expect("Should be able to read it back again");
        assert_eq!(i, result.0);
    }
}

#[test]
fn test_read_unsigned_overflow() {
    let mut readable = BytesMut::from(&[2u8 | CONTINUATION_BIT,
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
        1][..]);
    assert!(readable.read_unsigned().is_err());
}

#[test]
fn test_read_signed_overflow() {
    let mut readable = Bytes::from(&[2u8 | CONTINUATION_BIT,
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
        1][..]);
    assert!(readable.read_signed().is_err());
}

#[test]
fn test_read_multiple() {
    let mut readable = Bytes::from(&[2u8 | CONTINUATION_BIT, 1u8, 1u8][..]);

    assert_eq!(readable.read_unsigned().expect("Should read first number").0,
               130u64);
    assert_eq!(readable.read_unsigned().expect("Should read first number").0,
               1u64);
}