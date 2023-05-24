use std::fmt::Display;

use nom::{
  error::ParseError,
  multi::count,
  number::complete::{be_u16, be_u32, be_u8},
  sequence::tuple,
  IResult,
};

use crate::{get_constant_pool_ref, get_str_const};
use base::Parsable;
pub mod code;
pub mod linenumber_table;
pub mod stack_map_table;

pub const CODE_ATTRIBUTE_NAME: &str = "Code";
const CONSTANT_VALUE_ATTRIBUTE_NAME: &str = "ConstantValue";
const STACK_MAP_TABLE_ATTRIBUTE_NAME: &str = "StackMapTable";
const LINE_NUMBER_TABLE_ATTRIBUTE_NAME: &str = "LineNumberTable";
pub const SOURCE_FILE_ATTRIBUTE_NAME: &str = "SourceFile";
const DEPRECATED_ATTRIBUTE_NAME: &str = "Deprecated";

#[derive(Clone)]
pub struct AttributeInfo {
  attribute_name_index: u16,
  attribute_length: u32,
  // pub info_v: Vec<u8>,
  attribute_info: Attribute,
  // info: AttributeInfoType,
}

#[derive(Clone)]
pub enum Attribute {
  Code(code::CodeAttribute),
  Constant(ConstantValue),
  StackMapTable(stack_map_table::StackMapTable),
  LineNumberTable(linenumber_table::LineNumberTable),
  SourceFile(SourceFile),
  Deprecated,
  None,
}

pub fn parse_attributes<'a, E: ParseError<&'a [u8]>>(
  bytes: &'a [u8],
) -> IResult<&'a [u8], Vec<AttributeInfo>, E> {
  let (bytes, attribute_count) = be_u16(bytes)?;
  count(AttributeInfo::parse, attribute_count as usize)(bytes)
}

impl Parsable for AttributeInfo {
  fn parse<'a, E: ParseError<&'a [u8]>>(bytes: &'a [u8]) -> IResult<&'a [u8], Self, E> {
    let (bytes, (attribute_name_index, attribute_length)) = tuple((be_u16, be_u32))(bytes)?;
    let (bytes, info_v) = count(be_u8, attribute_length as usize)(bytes)?;
    // TODO ensure that attribute_length is correct
    let attr = if let Some(attr_str) =
      get_constant_pool_ref()[attribute_name_index as usize - 1].as_utf8()
    {
      // parse different attributes
      let ret =
        Self::parse_attribute::<nom::error::Error<_>>(&info_v, attr_str).map(|(_, attr)| attr);
      if let Err(_e) = ret {
        return Err(nom::Err::Error(E::from_error_kind(
          bytes,
          nom::error::ErrorKind::Tag,
        )));
      } else {
        let attr = ret.unwrap();
        attr
      }
    } else {
      Attribute::None
    };

    Ok((
      bytes,
      Self {
        attribute_name_index,
        attribute_length,
        attribute_info: attr,
      },
    ))
  }
}

impl AttributeInfo {
  fn parse_attribute<'a, E: ParseError<&'a [u8]>>(
    bytes: &'a [u8],
    attr_str: &str,
  ) -> IResult<&'a [u8], Attribute, E> {
    match attr_str {
      CODE_ATTRIBUTE_NAME => {
        let (bytes, code) = code::CodeAttribute::parse(bytes)?;
        Ok((bytes, Attribute::Code(code)))
      }
      CONSTANT_VALUE_ATTRIBUTE_NAME => {
        let (bytes, constant) = ConstantValue::parse(bytes)?;
        Ok((bytes, Attribute::Constant(constant)))
      }
      STACK_MAP_TABLE_ATTRIBUTE_NAME => {
        let (bytes, stack_map_table) = stack_map_table::StackMapTable::parse(bytes)?;
        Ok((bytes, Attribute::StackMapTable(stack_map_table)))
      }
      LINE_NUMBER_TABLE_ATTRIBUTE_NAME => {
        let (bytes, line_number_table) = linenumber_table::LineNumberTable::parse(bytes)?;
        Ok((bytes, Attribute::LineNumberTable(line_number_table)))
      }
      SOURCE_FILE_ATTRIBUTE_NAME => {
        let (bytes, source_file) = SourceFile::parse(bytes)?;
        Ok((bytes, Attribute::SourceFile(source_file)))
      }
      DEPRECATED_ATTRIBUTE_NAME => Ok((bytes, Attribute::Deprecated)),
      _ => Ok((bytes, Attribute::None)),
    }
  }

  pub fn type_filter(&self, attr_str: &str) -> bool {
    match attr_str {
      CODE_ATTRIBUTE_NAME => match &self.attribute_info {
        Attribute::Code(_) => true,
        _ => false,
      },
      CONSTANT_VALUE_ATTRIBUTE_NAME => match &self.attribute_info {
        Attribute::Constant(_) => true,
        _ => false,
      },
      STACK_MAP_TABLE_ATTRIBUTE_NAME => match &self.attribute_info {
        Attribute::StackMapTable(_) => true,
        _ => false,
      },
      LINE_NUMBER_TABLE_ATTRIBUTE_NAME => match &self.attribute_info {
        Attribute::LineNumberTable(_) => true,
        _ => false,
      },
      SOURCE_FILE_ATTRIBUTE_NAME => match &self.attribute_info {
        Attribute::SourceFile(_) => true,
        _ => false,
      },
      DEPRECATED_ATTRIBUTE_NAME => match &self.attribute_info {
        Attribute::Deprecated => true,
        _ => false,
      },
      _ => false,
    }
  }

  pub fn get_sourcefile(&self) -> Option<&str> {
    match &self.attribute_info {
      Attribute::SourceFile(source_file) => Some(source_file.get_sourcefile()),
      _ => None,
    }
  }

  pub fn name(&self) -> &str {
    get_constant_pool_ref()[self.attribute_name_index as usize - 1]
      .as_utf8()
      .unwrap()
  }
}

impl Display for AttributeInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // write!(f, "attribute_name_index: {}", self.attribute_name_index)?;
    write!(f, "{}", self.attribute_info)
  }
}

impl Display for Attribute {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Attribute::Code(code) => write!(f, "Code: {}", code),
      Attribute::Constant(constant) => write!(f, "Constant: {}", constant),
      Attribute::StackMapTable(stack_map_table) => {
        write!(f, "StackMapTable: {}", stack_map_table)
      }
      Attribute::LineNumberTable(line_number_table) => {
        write!(f, "LineNumberTable: {}", line_number_table)
      }
      Attribute::SourceFile(source_file) => write!(f, "SourceFile: {}", source_file),
      Attribute::Deprecated => write!(f, "Deprecated"),
      Attribute::None => write!(f, "None"),
    }
  }
}

#[derive(Clone)]
pub struct ConstantValue {
  constantvalue_index: u16,
}

impl Display for ConstantValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{{constantvalue_index: {}}}", self.constantvalue_index)
  }
}

impl Parsable for ConstantValue {
  fn parse<'a, E: ParseError<&'a [u8]>>(bytes: &'a [u8]) -> IResult<&'a [u8], Self, E> {
    let (bytes, constantvalue_index) = be_u16(bytes)?;
    Ok((
      bytes,
      Self {
        constantvalue_index,
      },
    ))
  }
}

#[derive(Clone)]
pub struct SourceFile {
  sourcefile_index: u16,
}

impl SourceFile {
  pub fn get_sourcefile<'a>(&self) -> &'a str {
    get_str_const(self.sourcefile_index as usize - 1)
  }
}

impl Parsable for SourceFile {
  fn parse<'a, E: nom::error::ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> nom::IResult<&'a [u8], Self, E>
  where
    Self: Sized,
  {
    let (bytes, sourcefile_index) = be_u16(bytes)?;
    Ok((bytes, Self { sourcefile_index }))
  }
}

impl Display for SourceFile {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{{sourcefile: {}}}", self.get_sourcefile())
  }
}
