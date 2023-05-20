use std::fmt::Display;

use crate::{
  access_flag::AccessFlags,
  attribute::{parse_attributes, AttributeInfo, SOURCE_FILE_ATTRIBUTE_NAME},
  constant_pool::ConstantPoolInfo,
  filed::FieldInfo,
  method::MethodInfo,
  ui::RenderSource,
  Parsable,
};
use nom::{error::ParseError, multi::count, number::complete::*, sequence::tuple, IResult};

pub struct ClassFile {
  magic: u32,
  minor_version: u16,
  major_version: u16,
  constant_pool_count: u16,
  constant_pool: Vec<ConstantPoolInfo>,
  access_flags: AccessFlags,
  this_class: u16,
  super_class: u16,
  interfaces: Vec<u16>,
  fields: Vec<FieldInfo>,
  methods: Vec<MethodInfo>,
  attributes: Vec<AttributeInfo>,
}

impl ClassFile {
  fn parse_constant_pool<'a, E: ParseError<&'a [u8]>>(
    bytes: &'a [u8],
    pool_count: u16,
  ) -> IResult<&'a [u8], Vec<ConstantPoolInfo>, E> {
    let mut pool_count = pool_count - 1;
    let mut m_bytes = bytes;
    let mut constant_pool = Vec::with_capacity(pool_count as usize);
    while pool_count > 0 {
      let (bytes, constant_pool_info) = ConstantPoolInfo::parse(m_bytes)?;
      m_bytes = bytes;
      if constant_pool_info.is_double_size() {
        pool_count -= 2;
        constant_pool.push(constant_pool_info);
        constant_pool.push(ConstantPoolInfo::new_empty());
      } else {
        pool_count -= 1;
        constant_pool.push(constant_pool_info);
      }
    }
    Ok((m_bytes, constant_pool))
  }

  fn parse_fields<'a, E: ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> IResult<&'a [u8], Vec<FieldInfo>, E> {
    let (bytes, fields) = be_u16(bytes)?;
    count(FieldInfo::parse, fields as usize)(bytes)
  }

  fn parse_methods<'a, E: ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> IResult<&'a [u8], Vec<MethodInfo>, E> {
    let (bytes, methods) = be_u16(bytes)?;
    count(MethodInfo::parse, methods as usize)(bytes)
  }

  pub fn parse_from_u8<'a>(bytes: &'a [u8]) -> Result<Self, crate::error::Error> {
    Self::_parse_from_u8::<nom::error::Error<_>>(bytes)
      .map(|(_, class)| class)
      .map_err(|e| crate::error::Error::from(e))
  }

  pub fn _parse_from_u8<'a, E: ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> IResult<&'a [u8], Self, E> {
    let (bytes, (magic, minor_version, major_version, constant_pool_count)) =
      tuple((be_u32, be_u16, be_u16, be_u16))(bytes)?;
    if magic != 0xCAFEBABE {
      log::error!("magic number is not 0xCAFEBABE");
      return Err(nom::Err::Error(E::from_error_kind(
        bytes,
        nom::error::ErrorKind::Tag,
      )));
    }
    let (bytes, constant_pool) = Self::parse_constant_pool(bytes, constant_pool_count)?;
    unsafe { crate::CONSTANT_POOL_REF = constant_pool.clone() };
    let (bytes, (access_flags, this_class, super_class, interfaces_count)) =
      tuple((be_u16, be_u16, be_u16, be_u16))(bytes)?;
    let (bytes, interfaces) = nom::multi::count(be_u16, interfaces_count as usize)(bytes)?;
    let (bytes, fields) = Self::parse_fields(bytes)?;
    let (bytes, methods) = Self::parse_methods(bytes)?;

    let (bytes, attributes) = parse_attributes(bytes)?;
    log::info!("pass");

    Ok((
      bytes,
      ClassFile {
        magic,
        minor_version,
        major_version,
        constant_pool_count,
        constant_pool,
        access_flags: AccessFlags::new_class_flag(access_flags),
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
        attributes,
      },
    ))
  }

  fn source_file_name(&self) -> String {
    let source_file = self
      .attributes
      .iter()
      .filter(|attr| attr.type_filter(SOURCE_FILE_ATTRIBUTE_NAME))
      .map(|attr| attr.get_sourcefile())
      .next();
    if let Some(source_file) = source_file {
      if let Some(file_name) = source_file {
        return file_name.to_string();
      }
    }
    return "Unknown".to_string();
  }
}

impl Display for ClassFile {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "magic: 0x{:08x}\nversion: {}.{}\naccess_flags: {}\nconst pool({}):\n",
      self.magic,
      self.major_version,
      self.minor_version,
      self.access_flags,
      self.constant_pool_count
    )?;
    for (_, info) in self.constant_pool.iter().enumerate() {
      write!(f, "\t{}\n", info)?;
    }
    write!(
      f,
      "this class: {}\nsuper class: {}\n",
      self.this_class, self.super_class
    )?;
    write!(f, "interfaces({}):\n", self.interfaces.len())?;
    for (_, interface) in self.interfaces.iter().enumerate() {
      write!(f, "\t{}\n", interface)?;
    }
    write!(f, "fields({}):\n", self.fields.len())?;
    for (_, field) in self.fields.iter().enumerate() {
      write!(f, "\t{}\n", field)?;
    }
    write!(f, "methods({}):\n", self.methods.len())?;
    for (_, method) in self.methods.iter().enumerate() {
      write!(f, "\t{}\n", method)?;
    }
    write!(f, "attributes({}):\n", self.attributes.len())?;
    for (_, attribute) in self.attributes.iter().enumerate() {
      write!(f, "\t{}\n", attribute)?;
    }
    Ok(())
  }
}

impl RenderSource for ClassFile {
  fn render_file_info(&self) -> Vec<String> {
    vec![
      format!("magic: 0x{:08x}", self.magic),
      format!("version: {}.{}", self.major_version, self.minor_version),
      format!("source file: {}", self.source_file_name()),
    ]
  }

  fn render_class_info(&self) -> Vec<String> {
    vec![
      format!("this class: {}", self.this_class),
      format!("super class: {}", self.super_class),
      format!("access_flags: {}", self.access_flags),
    ]
  }

  fn render_constant_pool(&self) -> Vec<String> {
    let mut result = vec![];
    for (i, info) in self.constant_pool.iter().enumerate() {
      result.push(format!("#{}: {}", i + 1, info));
    }
    result
  }

  fn render_interfaces(&self) -> Vec<String> {
    self
      .interfaces
      .iter()
      .map(|interface| crate::get_str_const(*interface as usize - 1).to_string())
      .collect()
  }

  fn render_fields(&self) -> Vec<String> {
    self
      .fields
      .iter()
      .map(|field| field.name().to_string())
      .collect()
  }

  fn render_methods(&self) -> Vec<String> {
    self
      .methods
      .iter()
      .map(|method| method.name().to_string())
      .collect::<Vec<String>>()
  }

  fn render_methods_verbose(&self) -> Vec<&MethodInfo> {
    self.methods.iter().collect::<Vec<&MethodInfo>>()
  }

  fn render_attributes(&self) -> Vec<String> {
    self
      .attributes
      .iter()
      .map(|attr| attr.name().to_string())
      .collect()
  }
}
