use std::fmt::Display;

use nom::{error::ParseError, number::complete::be_u16, sequence::tuple, IResult};

use crate::{
  access_flag::AccessFlags,
  attribute::{parse_attributes, AttributeInfo},
  get_str_const, Parsable,
};

pub struct MethodInfo {
  access_flags: AccessFlags,
  name_index: u16,
  descriptor_index: u16,
  pub attributes: Vec<AttributeInfo>,
}

impl Parsable for MethodInfo {
  fn parse<'a, E: ParseError<&'a [u8]>>(bytes: &'a [u8]) -> IResult<&'a [u8], Self, E> {
    let (bytes, (access_flags, name_index, descriptor_index)) =
      tuple((be_u16, be_u16, be_u16))(bytes)?;
    let (bytes, attributes) = parse_attributes(bytes)?;

    Ok((
      bytes,
      Self {
        access_flags: AccessFlags::new_method_flag(access_flags),
        name_index,
        descriptor_index,
        attributes,
      },
    ))
  }
}
impl MethodInfo {
  pub fn name(&self) -> &str {
    get_str_const(self.name_index as usize - 1)
  }
}

impl Display for MethodInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "access_flags: {} name_index: {} descriptor_index: {}",
      self.access_flags, self.name_index, self.descriptor_index
    )?;
    if self.attributes.len() > 0 {
      write!(f, " attributes({}):", self.attributes.len())?;
    }
    for attribute in &self.attributes {
      write!(f, " {}", attribute)?;
    }

    Ok(())
  }
}
