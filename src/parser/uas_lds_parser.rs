//! UAS Datalink Local Data Set Parser
//! Author: Jeramy Singleton
//! Email: jeramyRR@gmail.com
//! Since: 18 November 2017
//!
//! The UAS Datalink Local Data Set (LDS)
//!
//! A LDS can use a 1, 2 or 4 byte key with a 1, 2, 4 byte, or BER encoded
//! length.  The UAS Local Data Set uses BER encoded lengths and BER'OID encoded
//! tags.
//!
//! ---------------------------------------------------------------------------
//! Local Sets:
//!
//! A local set allows efficient use of the KLV encoding as it tries to compact
//! use of the metadata in as few large KLVs as possible.  KLV wouldn't be
//! terribly efficient if we used a 16 byte Key and 1 byte Length only to send
//! a 2 byte value.
//!
//! Local sets group multiple KLVs inside of the Value portion of a larger KLV,
//! but instead of each inner KLV having a 16 byte key they have small 1 or 2
//! byte Tags.
//!
//! This will look something like:
//!
//! +-----------+----------+-----------------------------------------------+
//! |           |          |                     Value                     |
//! |    Key    |  Length  +-----+---+-----+-----+---+-----+-----+---+-----+
//! |           |          | Tag | L |  V  | TAG | L |  V  | TAG | L |  V  |
//! +-----------+----------+-----+---+-----+-----+---+-----+-----+---+-----+
//!
//! ---------------------------------------------------------------------------
//! UAS Local Set 16-byte Universal Key:
//!
//! 06 0E 2B 34 - 02 0B 01 01 – 0E 01 03 01 - 01 00 00 00
//!
//! ---------------------------------------------------------------------------
//! References:
//!
//! # [ST0601.8](http://www.gwg.nga.mil/misb/docs/standards/ST0601.8.pdf)
//! # [EG0104.4](http://www.gwg.nga.mil/misb/docs/eg/EG0104.4.pdf)

pub struct Tlv {
  tag: u8,
  length: u8,
  value: Vec<u8>
}