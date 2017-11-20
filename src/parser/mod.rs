//! KLV Parsing module
//! Author: Jeramy Singleton
//! Email: jeramyRR@gmail.com
//! Since: 17 November 2017
//!
//! Key Length Value (KLV) is a way for video to efficiently include metadata
//! that will have the least impact on streaming latency.
//!
//! This module currently focuses on those KLVs that are encoded using
//! ISO/IEC 8825-1 BASIC ENCODING RULES (BER)
//!
//! BER defines an encoding format that describe what kind of data is
//! being transmitted (string, boolean, integers, floats, etc) (this
//! is known as the key), how large the data is (known as length), and
//! is then followed by the actual data (known as the value).
//!
//! This looks like the diagram below:
//!
//! +----------+-----------+----------------+
//! |    Key   |   Length  |    Contents    |
//! | 16 Bytes | 1-n Bytes | "Length" bytes |
//! +----------+-----------+----------------+
//!
//! There are two forms of BER:
//! short: Length is defined by 1 byte
//! long : Length can be defined by multiple bytes
//!
//! Short form can be identified by finding a "0" in the Most Significant
//! Byte (MSB).
//! Long form can be identified by finding a "1" in the MSB. The remaining
//! bytes indicate the how many more bytes will be used for the length.
//!
//! This basically equates to short form length value being 127 or less.
//!
//! ---------------------------------------------------------------------------
//! Key:
//!
//! The Key is a Universal Label (UL) (defined by SMPTE 298M), and is always
//! 16 bytes long.  It represents a string that can be found in the
//! registry specific to the metadata you are parsing.
//!
//! A key always starts with the byte 0x06.  This is called the Object
//! Identifier (OID).
//!
//! The second byte is the Length of the key, and will always be 0x0E.  As
//! we indicated above, the key is always 16 bytes.  Take the first two bytes
//! (the OID, and this Length byte) and subtract 2 from 0x0E (which is 14), and
//! you get 16.
//!
//! The next two bytes represent the organization responsible for the standard
//! and will always be 0x2B 0x34 (SMPTE).
//!
//! The next four bytes tell us which document to reference for UL strings, and
//! which version of the document for which to look.  You can think of this
//! document as a registry.
//!
//! ---------------------------------------------------------------------------
//! Length:
//!
//! The Length value tells us how many bytes make up the value part of the KLV.
//! The Length can never be more than nine bytes, but realistically will
//! probably be in the one to four range.
//!
//! Note that the Length value, since it is BER encoded itself may be a little
//! tricky at first to decode.
//!
//! ---------------------------------------------------------------------------
//! Value:
//!
//! At first glance there doesn't seem to be anything special about this part
//! of the KLV.  The data is packed in byte order and is exactly as long as the
//! LENGTH portion of the KLV dictates.
//!

//!
//! ---------------------------------------------------------------------------
//! References:
//!
//! # [ST0601.8](http://www.gwg.nga.mil/misb/docs/standards/ST0601.8.pdf)
//! # [Encoding to MXF](https://www.amwa.tv/downloads/whitepapers/encodingtoMXF.pdf)

const KEY_LENGTH: usize = 16;

#[derive(Debug)]
pub struct Klv {
  key: [u8; KEY_LENGTH],
  length: usize,
  value: Vec<u8>,
}

#[derive(Debug)]
pub struct Parser<'a> {
  input: &'a [u8],
  klvs: Vec<Klv>,
  cursor: usize,
}

impl<'a> Parser<'a> {
  pub fn new(input: &[u8]) -> Parser {
    Parser {
      input: input,
      klvs: Vec::new(),
      cursor: 0,
    }
  }

  fn has_next(&self) -> bool {
    self.cursor < self.input.len()
  }

  fn read_next(&mut self) -> Klv {
    let key: [u8; KEY_LENGTH] = self.get_key().expect("Unable to parse key");
    println!("key is: {:?}", key);
    let length: usize = self.get_length().expect("Unable to parse length");
    println!("length is: {}", length);
    let value: Vec<u8> = self.get_value(length).expect("Unable to parse value");

    let klv = Klv {
      key: key,
      length: length,
      value: value,
    };

    println!("klv: {:?}", klv);
    klv
  }

  fn increment_cursor(&mut self, bytes: usize) {
    self.cursor += bytes;
  }

  fn get_length(&mut self) -> Option<usize> {
    if self.cursor < self.input.len() {
      let byte = self.input[self.cursor];

      // here we have to check for BER short or long form.
      // short form will have a 0 as the MSB, so the value
      // of this byte will be 127 or less.
      if byte < 128 {
        self.increment_cursor(1);
        Some(byte as usize)
      } else {
        // we now know that the BER form is the long form.
        // We now have to check the remaining bytes to see
        // how many more bytes will contain the actual length
        // of the value portion of this packet.
        let bytes_length: usize = (byte - 127) as usize;
        let u8s_array: &[u8] = &self.input[self.cursor..self.cursor + bytes_length];
        self.increment_cursor(bytes_length);
        let num: u32 = u8s_to_u32(u8s_array);
        Some(num as usize)
        }
    } else {
      None
    }
  }

  fn get_key(&mut self) -> Option<[u8; KEY_LENGTH]> {
    let cursor_start_pos = self.cursor;
    let cursor_end_pos = self.cursor + KEY_LENGTH;

    println!("cursor_start: {}, cursor_end: {}", cursor_start_pos, cursor_end_pos);
    if self.cursor < self.input.len() && cursor_end_pos <= self.input.len() {
      self.increment_cursor(KEY_LENGTH);
      let input_slice: &[u8] = &self.input[cursor_start_pos..cursor_end_pos];
      let mut klv_array = [0u8; KEY_LENGTH];

      for (&x, p) in input_slice.iter().zip(klv_array.iter_mut()) {
        *p = x;
      }

      Some(klv_array)
    } else {
      None
    }
  }

  /// get_value is a little tricky.  In this function we need to separate out all
  /// the bytes specified by the length argument into more key lentgh values, or
  /// tag length values in this case.
  fn get_value(&mut self, length: usize) -> Option<Vec<u8>> {
    let cursor_start_pos = self.cursor;
    let cursor_end_pos = self.cursor + length;

    println!("cursor_start: {}, cursor_end: {}", cursor_start_pos, cursor_end_pos);
    if self.cursor < self.input.len() && cursor_end_pos <= self.input.len() {
      self.increment_cursor(length);
      let input_slice: &[u8] = &self.input[cursor_start_pos..cursor_end_pos];
      Some(input_slice.to_vec())
    } else {
      None
    }
  }
}

pub fn parse(bytes: &[u8]) -> Vec<Klv> {
  let mut klv_vec: Vec<Klv> = Vec::new();
  let mut parser: Parser = Parser::new(bytes);
  while parser.has_next() {
    let klv = parser.read_next();
    klv_vec.push(klv);
  }
  klv_vec
}

pub fn u8s_to_u32(bytes: &[u8]) -> u32 {
  let size: usize = bytes.len();
  match size {
    2 => two_u8s_to_u32(bytes),
    3 => three_u8s_to_u32(bytes),
    4 => four_u8s_to_u32(bytes),
    _ => panic!(format!("bytes array size must be between 2 and 4.  size was {}", size))
  }
}

/// Takes the first two bytes (u8) from the slice
/// and converts them to one u32 value.
pub fn two_u8s_to_u32(bytes: &[u8]) -> u32 {
  if bytes.len() >= 2 {
    (u32::from(bytes[1]) << 8) | (u32::from(bytes[0]))
  } else {
    panic!("bytes array was too small to convert to u32.  Needed at least two elements in the bytes array");
  }
}

pub fn three_u8s_to_u32(bytes: &[u8]) -> u32 {
  if bytes.len() >= 3 {
    (u32::from(bytes[2]) << 16) | (u32::from(bytes[1]) << 8) | (u32::from(bytes[0]))
  } else {
    panic!("bytes array was too small to convert to u32.  Needed at least three elements in the bytes array");
  }
}

pub fn four_u8s_to_u32(bytes: &[u8]) -> u32 {
  if bytes.len() >= 4 {
    (
      (u32::from(bytes[3]) << 24) |
      (u32::from(bytes[2]) << 16) |
      (u32::from(bytes[1]) << 8) |
      (u32::from(bytes[0]))
    )
  } else {
    panic!("bytes array was too small to convert to u32.  Needed at least four elements in the bytes array");
  }
}

#[test]
fn test_parser() {
  let data = include_bytes!("../../test/assets/out.klv");
  let klvs: Vec<Klv> = parse(&data.to_vec());

  println!("{:?}", klvs);
}

#[test]
fn test_two_bytes_to_u32() {
  //                       8          7         6         5         4        3        2        1
  // -------------------------------------------------------------------------------------------
  // 16 bytes:         32768      16384      8192      4096      2048     1024      512      256
  // 124:                  0          1         1         1         1        1        0        0
  //
  // 8 bytes:            128         64        32        16         8        4        2        1
  // 45:                   0          0         1         0         1        1        0        1
  let expected_num: u32 = 31789;

  let array: [u8; 2] = [45, 124];

  let actual_num: u32 = two_u8s_to_u32(&array);

  assert_eq!(actual_num, expected_num);
}

#[test]
fn test_three_bytes_to_u32() {
  //                       8          7         6         5         4        3        2        1
  // -------------------------------------------------------------------------------------------
  // 24 bytes:       8388608    4194304   2097152   1048576    524288   262144   131072    65536
  // 101:                  0          1         1         0         0        1        0        1
  //
  // 16 bytes:         32768      16384      8192      4096      2048     1024      512      256
  // 124:                  0          1         1         1         1        1        0        0
  //
  // 8 bytes:            128         64        32        16         8        4        2        1
  // 45:                   0          0         1         0         1        1        0        1
  let expected_num: u32 = 6650925;

  let array: [u8; 3] = [45, 124, 101];
  let actual_num: u32 = three_u8s_to_u32(&array);

  assert_eq!(actual_num, expected_num);
}

#[test]
fn test_four_bytes_to_u32() {
  //                       8          7         6         5         4        3        2        1
  // -------------------------------------------------------------------------------------------
  // 32 bytes:    2147483648 1073741824 536870912 268435456 134217728 67108864 33554432 16777216
  // 12:                   0          0         0         0         1        1        0        0
  //
  // 24 bytes:       8388608    4194304   2097152   1048576    524288   262144   131072    65536
  // 101:                  0          1         1         0         0        1        0        1
  //
  // 16 bytes:         32768      16384      8192      4096      2048     1024      512      256
  // 124:                  0          1         1         1         1        1        0        0
  //
  // 8 bytes:            128         64        32        16         8        4        2        1
  // 45:                   0          0         1         0         1        1        0        1
  let expected_num: u32 = 207977517;

  let array: [u8; 4] = [45, 124, 101, 12];
  let actual_num: u32 = four_u8s_to_u32(&array);

  assert_eq!(actual_num, expected_num);
}
