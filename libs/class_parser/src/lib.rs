use error::Error;
use raw_class::ClassFile;

mod access_flag;
pub mod attribute;
mod constant_pool;
pub mod error;
mod filed;
mod method;
mod opcodes;
pub mod raw_class;

pub fn add(left: usize, right: usize) -> usize {
  left + right
}

pub fn parse<'a>(bytes: &'a [u8]) -> Result<ClassFile, Error> {
  ClassFile::parse_from_u8(bytes)
}

static mut CONSTANT_POOL_REF: Vec<constant_pool::ConstantPoolInfo> = vec![];

pub fn get_constant_pool_ref() -> &'static Vec<constant_pool::ConstantPoolInfo> {
  unsafe { &CONSTANT_POOL_REF }
}

pub fn get_str_const<'a>(index: usize) -> &'a str {
  get_constant_pool_ref()[index].as_utf8().unwrap()
}

trait Parsable {
  fn parse<'a, E: nom::error::ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> nom::IResult<&'a [u8], Self, E>
  where
    Self: Sized;

  fn parse_from_u8<'a>(bytes: &'a [u8]) -> Result<Self, crate::error::Error>
  where
    Self: Sized,
  {
    Self::parse::<nom::error::Error<_>>(bytes)
      .map(|(_, v)| v)
      .map_err(|e| e.into())
  }
}

#[cfg(test)]
mod tests {
  use crate::raw_class::ClassFile;

  use super::*;

  #[test]
  fn it_works() {
    let result = add(2, 2);
    assert_eq!(result, 4);
  }

  #[test]
  fn test_parse() {
    let data = String::from("");
    let res = ClassFile::parse_from_u8(data.as_bytes());
    assert_ne!(res.is_err(), true)
  }
}
