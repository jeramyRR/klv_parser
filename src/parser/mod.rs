/// KLV Parsing module
/// Author: Jeramy Singleton
/// Email: jeramyRR@gmail.com
/// Since: 17 November 2017
///
/// Key Length Value (KLV) is a way for video to efficiently include metadata
/// that will have the least impact on streaming latency.
///
/// This module currently focuses on those KLVs that are encoded using
/// ISO/IEC 8825-1 BASIC ENCODING RULES (BER)
///
/// BER defines an encoding format that describe what kind of data is
/// being transmitted (string, boolean, integers, floats, etc) (this
/// is known as the key), how large the data is (known as length), and
/// is then followed by the actual data (known as the value).
///
/// This looks like the diagram below:
///
/// +----------+-----------+----------------+
/// |    Key   |   Length  |    Contents    |
/// | 16 Bytes | 1-n Bytes | "Length" bytes |
/// +----------+-----------+----------------+
///
/// There are two forms of BER:
/// short: Length is defined by 1 byte
/// long : Length can be defined by multiple bytes
///
/// Short form can be identified by finding a "0" in the Most Significant
/// Byte (MSB).
/// Long form can be identified by finding a "1" in the MSB. The remaining
/// bytes indicate the how many more bytes will be used for the length.
///
/// This basically equates to short form length value being 127 or less.
///
/// ---------------------------------------------------------------------------
/// Key:
///
/// The Key is a Universal Label (UL) (defined by SMPTE 298M), and is always
/// 16 bytes long.  It represents a string that can be found in the
/// registry specific to the metadata you are parsing.
///
/// A key always starts with the byte 0x06.  This is called the Object
/// Identifier (OID).
///
/// The second byte is the Length of the key, and will always be 0x0E.  As
/// we indicated above, the key is always 16 bytes.  Take the first two bytes
/// (the OID, and this Length byte) and subtract 2 from 0x0E (which is 14), and
/// you get 16.
///
/// The next two bytes represent the organization responsible for the standard
/// and will always be 0x2B 0x34 (SMPTE).
///
/// The next four bytes tell us which document to reference for UL strings, and
/// which version of the document for which to look.  You can think of this
/// document as a registry.
///
/// ---------------------------------------------------------------------------
/// Length:
///
/// The Length value tells us how many bytes make up the value part of the KLV.
/// The Length can never be more than nine bytes, but realistically will
/// probably be in the one to four range.
///
/// Note that the Length value, since it is BER encoded itself may be a little
/// tricky at first to decode.
///
/// ---------------------------------------------------------------------------
/// Value:
///
/// At first glance there doesn't seem to be anything special about this part
/// of the KLV.  The data is packed in byte order and is exactly as long as the
/// LENGTH portion of the KLV dictates.
///
/// ---------------------------------------------------------------------------
/// Local Sets:
///
/// A local set allows efficient use of the KLV encoding as it tries to compact
/// use of the metadata in as few large KLVs as possible.  KLV wouldn't be
/// terribly efficient if we used a 16 byte Key and 1 byte Length only to send
/// a 2 byte value.
///
/// Local sets group multiple KLVs inside of the Value portion of a larger KLV,
/// but instead of each inner KLV having a 16 byte key they have small 1 or 2
/// byte Tags.
///
/// This will look something like:
///
/// +-----------+----------+-----------------------------------------------+
/// |           |          |                     Value                     |
/// |    Key    |  Length  +-----+---+-----+-----+---+-----+-----+---+-----+
/// |           |          | Tag | L |  V  | TAG | L |  V  | TAG | L |  V  |
/// +-----------+----------+-----+---+-----+-----+---+-----+-----+---+-----+
///
/// ---------------------------------------------------------------------------
/// References:
///
/// # http://www.gwg.nga.mil/misb/docs/standards/ST0601.8.pdf
/// # https://www.amwa.tv/downloads/whitepapers/encodingtoMXF.pdf

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
    let length: usize = self.get_length().expect("Unable to parse length");
    let value: Vec<u8> = self.get_value(length).expect("Unable to parse value");

    let klv = Klv {
      key: key,
      length: len,
      value: value,
    };

    println!("klv: {:?}", klv);
    klv
  }

  fn increment_cursor(&mut self, bytes: usize) {
    self.cursor += bytes;
  }

  fn get_length(&mut self) -> Option<usize> {
    match self.cursor < self.input.len() {
      true => {
        let byte = self.input[self.cursor];
        self.increment_cursor(1);
        Some(byte as usize)
      }
      false => None,
    }
  }

  fn get_key(&mut self) -> Option<[u8; KEY_LENGTH]> {
    let cursor_start_pos = self.cursor;
    let cursor_end_pos = self.cursor + KEY_LENGTH;

    println!("cursor_start: {}, cursor_end: {}", cursor_start_pos, cursor_end_pos);
    match self.cursor < self.input.len() && cursor_end_pos <= self.input.len() {
      true => {
        self.increment_cursor(KEY_LENGTH);
        let input_slice: &[u8] = &self.input[cursor_start_pos..cursor_end_pos];
        let mut klv_array = [0u8; KEY_LENGTH];

        for (&x, p) in input_slice.iter().zip(klv_array.iter_mut()) {
          *p = x;
        }

        Some(klv_array)
      }
      false => None,
    }
  }

  fn get_value(&mut self, length: usize) -> Option<Vec<u8>> {
    let cursor_start_pos = self.cursor;
    let cursor_end_pos = self.cursor + length;

    println!("cursor_start: {}, cursor_end: {}", cursor_start_pos, cursor_end_pos);
    match self.cursor < self.input.len() && cursor_end_pos <= self.input.len() {
      true => {
        self.increment_cursor(length);
        let input_slice: &[u8] = &self.input[cursor_start_pos..cursor_end_pos];
        Some(input_slice.to_vec())
      }
      false => None,
    }
  }
}

pub fn parse(bytes: &Vec<u8>) -> Vec<Klv> {
  let mut klv_vec: Vec<Klv> = Vec::new();
  let mut parser: Parser = Parser::new(bytes.as_slice());
  while parser.has_next() {
    let klv = parser.read_next();
    klv_vec.push(klv);
  }
  klv_vec
}

#[test]
fn test_parser() {
  let data = include_bytes!("../test/assets/out.klv");
  let klvs: Vec<Klv> = parse(&data.to_vec());

  println!("{:?}", klvs);
}
