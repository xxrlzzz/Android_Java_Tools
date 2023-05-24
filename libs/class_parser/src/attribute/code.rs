use std::fmt::Display;

use nom::{
  error::ParseError,
  multi::count,
  number::complete::{be_u16, be_u32, be_u8},
  sequence::tuple,
  IResult,
};

use crate::opcodes::CodeInfo;

use base::Parsable;

use super::{parse_attributes, AttributeInfo};

#[derive(Clone)]
pub struct CodeAttribute {
  max_stack: u16,
  max_locals: u16,
  code_length: u32,
  code: Vec<CodeInfo>,
  exception_table: Vec<ExceptionTable>,
  attributes: Vec<AttributeInfo>,
}
#[derive(Debug, Clone, Copy)]
pub struct ExceptionTable {
  start_pc: u16,
  end_pc: u16,
  handler_pc: u16,
  catch_type: u16,
}

fn parse_code_infos<'a, E: ParseError<&'a [u8]>>(
  bytes: &'a [u8],
) -> IResult<&'a [u8], Vec<CodeInfo>, E> {
  let mut code_bytes: &[u8] = bytes;
  let mut code_infos = vec![];
  while code_bytes.len() > 0 {
    let (bytes, code_info) = CodeInfo::parse(code_bytes)?;
    code_bytes = bytes;
    code_infos.push(code_info.clone());
  }
  Ok((bytes, code_infos))
}

impl Parsable for CodeAttribute {
  fn parse<'a, E: ParseError<&'a [u8]>>(bytes: &'a [u8]) -> IResult<&'a [u8], Self, E> {
    let (bytes, (max_stack, max_locals, code_length)) = tuple((be_u16, be_u16, be_u32))(bytes)?;
    let (bytes, code) = count(be_u8, code_length as usize)(bytes)?;
    let (bytes, exception_table_length) = be_u16(bytes)?;
    let (bytes, exception_table) =
      count(ExceptionTable::parse, exception_table_length as usize)(bytes)?;
    let (bytes, attributes) = parse_attributes(bytes)?;
    let code_infos =
      parse_code_infos::<nom::error::Error<_>>(&code).map(|(_, code_infos)| code_infos);
    if let Err(_e) = code_infos {
      return Err(nom::Err::Error(E::from_error_kind(
        bytes,
        nom::error::ErrorKind::Tag,
      )));
    }
    Ok((
      bytes,
      Self {
        max_stack,
        max_locals,
        code_length,
        code: code_infos.unwrap(),
        exception_table,
        attributes,
      },
    ))
  }
}

impl ExceptionTable {
  fn parse<'a, E: ParseError<&'a [u8]>>(bytes: &'a [u8]) -> IResult<&'a [u8], Self, E> {
    let (bytes, (start_pc, end_pc, handler_pc, catch_type)) =
      tuple((be_u16, be_u16, be_u16, be_u16))(bytes)?;
    Ok((
      bytes,
      Self {
        start_pc,
        end_pc,
        handler_pc,
        catch_type,
      },
    ))
  }
}

impl Display for CodeAttribute {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{{max_stack: {}, max_locals: {}, code_length: {}}}",
      self.max_stack, self.max_locals, self.code_length
    )?;
    write!(f, "\ncode: ")?;
    for code in &self.code {
      write!(f, "{} ", code)?;
    }
    Ok(())
  }
}
impl CodeAttribute {}
