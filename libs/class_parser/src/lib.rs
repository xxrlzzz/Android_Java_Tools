use base::error::Error;
use raw_class::ClassFile;

pub mod attribute;
mod constant_pool;
mod filed;
mod method;
mod opcodes;
pub mod raw_class;

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

#[cfg(test)]
mod tests {
  use crate::raw_class::ClassFile;

  #[test]
  fn test_parse() {
    let data = String::from("");
    let res = ClassFile::parse_from_u8(data.as_bytes());
    assert_ne!(res.is_err(), true)
  }
}
