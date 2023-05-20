use std::fmt::Display;

use nom::{error::ParseError, number::complete::be_u16, sequence::tuple, IResult};

use crate::{
  access_flag::AccessFlags,
  attribute::{parse_attributes, AttributeInfo},
  Parsable,
};

pub struct FieldInfo {
  access_flags: AccessFlags,
  name_index: u16,
  descriptor_index: u16,
  attributes: Vec<AttributeInfo>,
}

impl Parsable for FieldInfo {
  fn parse<'a, E: ParseError<&'a [u8]>>(bytes: &'a [u8]) -> IResult<&'a [u8], Self, E> {
    let (bytes, (access_flags, name_index, descriptor_index)) =
      tuple((be_u16, be_u16, be_u16))(bytes)?;
    let (bytes, attributes) = parse_attributes(bytes)?;
    Ok((
      bytes,
      Self {
        access_flags: AccessFlags::new_filed_flag(access_flags),
        name_index,
        descriptor_index,
        attributes,
      },
    ))
  }
}
impl FieldInfo {
  pub fn name(&self) -> &str {
    crate::get_str_const(self.name_index as usize - 1)
  }
}

impl Display for FieldInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "access_flags: {}\tname_index: {}\tdescriptor_index: {}",
      self.access_flags, self.name_index, self.descriptor_index,
    )?;
    if self.attributes.len() > 0 {
      write!(f, "\tattributes({}):", self.attributes.len())?;
    }
    for attribute in &self.attributes {
      write!(f, " {}", attribute)?;
    }

    Ok(())
  }
}
