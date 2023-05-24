use std::fmt::Display;

use nom::{error::ParseError, multi::count, number::complete::be_u16, IResult};

use base::Parsable;

#[derive(Clone)]
pub struct LineNumberTableAttribute {
  line_number_table_length: u16,
  line_number_table: Vec<LineNumberTable>,
}

#[derive(Clone)]
pub struct LineNumberTable {
  start_pc: u16,
  line_number: u16,
}

impl Parsable for LineNumberTableAttribute {
  fn parse<'a, E: ParseError<&'a [u8]>>(bytes: &'a [u8]) -> IResult<&'a [u8], Self, E> {
    let (bytes, line_number_table_length) = be_u16(bytes)?;
    let (bytes, line_number_table) =
      count(LineNumberTable::parse, line_number_table_length as usize)(bytes)?;
    Ok((
      bytes,
      Self {
        line_number_table_length,
        line_number_table,
      },
    ))
  }
}

impl Parsable for LineNumberTable {
  fn parse<'a, E: nom::error::ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> nom::IResult<&'a [u8], Self, E>
  where
    Self: Sized,
  {
    let (bytes, (start_pc, line_number)) = nom::sequence::tuple((be_u16, be_u16))(bytes)?;
    Ok((
      bytes,
      Self {
        start_pc,
        line_number,
      },
    ))
  }
}

impl Display for LineNumberTableAttribute {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "line_number_table({}): ", self.line_number_table_length)?;
    for line_number_table in &self.line_number_table {
      write!(f, "{} ", line_number_table)?;
    }
    Ok(())
  }
}

impl Display for LineNumberTable {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{{start_pc: {}, line_number: {}}}",
      self.start_pc, self.line_number
    )
  }
}
