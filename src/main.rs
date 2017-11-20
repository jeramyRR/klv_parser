mod parser;

use std::fs::File;
use std::io::Read;

fn main() {
  let file_name: &str = "/Users/jeramy/dev/rust/klv_parser/test/assets/out.klv";
  let mut file = File::open(file_name).expect("file not found");

  let mut buffer: Vec<u8> = Vec::new();
  let _result = file.read_to_end(&mut buffer);

  let klvs = parser::parse(&buffer);
}

