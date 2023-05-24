use std::fmt::Display;

use base::{access_flag::AccessFlags, error::Error, Parsable};
use nom::{
  error::ParseError,
  multi::count,
  number::complete::{le_u16, le_u32},
  sequence::tuple,
  IResult, Slice,
};

use crate::{
  get_field_id, get_method_id, get_type_id, get_type_id_ref,
  leb128::parse_uleb128_nom,
  raw_dex::{FieldIdItem, MethodIdItem, TypeIdItem, TypeList},
};

pub struct ClassDefItem {
  class_idx: u32,
  class: TypeIdItem,
  access_flags: AccessFlags,
  superclass_idx: u32,
  superclass: Option<TypeIdItem>,
  interfaces_off: u32,
  interfaces: Option<TypeList>,
  source_file_idx: Option<u32>,
  annotations_off: u32,
  class_data_off: u32,
  static_values_off: u32,

  class_data_item: Option<ClassDataItem>,
}

pub struct ClassDataItem {
  static_fields: Vec<EncodedField>,
  instance_fields: Vec<EncodedField>,
  direct_methods: Vec<EncodedMethod>,
  virtual_methods: Vec<EncodedMethod>,
}

impl ClassDataItem {
  pub fn parse_from_u8<'a>(bytes: &'a [u8], origin_bytes: &'a [u8]) -> Result<Self, Error> {
    Self::parse(bytes, origin_bytes)
      .map(|(_, class_data_item)| class_data_item)
      .map_err(|e| e.into())
  }

  pub fn parse<'a, E: ParseError<&'a [u8]>>(
    bytes: &'a [u8],
    origin_bytes: &'a [u8],
  ) -> IResult<&'a [u8], Self, E> {
    let (
      bytes,
      (static_fields_size, instance_fields_size, direct_methods_size, virtual_methods_size),
    ) = tuple((
      parse_uleb128_nom,
      parse_uleb128_nom,
      parse_uleb128_nom,
      parse_uleb128_nom,
    ))(bytes)?;
    let mut m_bytes = bytes;
    let mut cur_offset = 0;
    let mut static_fields = vec![];
    for _ in 0..static_fields_size {
      let (bytes, mut field) = EncodedField::parse(m_bytes)?;
      cur_offset = field.field_idx_diff + cur_offset;
      field.field = get_field_id(cur_offset as usize);
      static_fields.push(field);
      m_bytes = bytes;
    }
    cur_offset = 0;
    let mut instance_fields = vec![];
    for _ in 0..instance_fields_size {
      let (bytes, mut field) = EncodedField::parse(m_bytes)?;
      cur_offset = field.field_idx_diff + cur_offset;
      field.field = get_field_id(cur_offset as usize);
      instance_fields.push(field);
      m_bytes = bytes;
    }
    cur_offset = 0;
    let mut direct_methods = vec![];
    for _ in 0..direct_methods_size {
      let (bytes, mut method) = EncodedMethod::parse(m_bytes)?;
      cur_offset = method.method_idx_diff + cur_offset;
      method.method = get_method_id(cur_offset as usize);

      let code_item = if method.code_off != 0 {
        let offset_bytes = origin_bytes.slice(method.code_off as usize..);
        let (_, code_item) = CodeItem::parse(offset_bytes)?;
        Some(code_item)
      } else {
        None
      };
      method.code_item = code_item;
      direct_methods.push(method);
      m_bytes = bytes;
    }
    cur_offset = 0;
    let mut virtual_methods = vec![];
    for _ in 0..virtual_methods_size {
      let (bytes, mut method) = EncodedMethod::parse(m_bytes)?;
      cur_offset = method.method_idx_diff + cur_offset;
      method.method = get_method_id(cur_offset as usize);

      let code_item = if method.code_off != 0 {
        let offset_bytes = origin_bytes.slice(method.code_off as usize..);
        let (_, code_item) = CodeItem::parse(offset_bytes)?;
        Some(code_item)
      } else {
        None
      };
      method.code_item = code_item;
      virtual_methods.push(method);
      m_bytes = bytes;
    }
    let ret = Self {
      static_fields,
      instance_fields,
      direct_methods,
      virtual_methods,
    };
    Ok((m_bytes, ret))
  }
}

pub struct EncodedField {
  field_idx_diff: u32,
  access_flags: AccessFlags,
  field: FieldIdItem,
}

impl Parsable for EncodedField {
  fn parse<'a, E: nom::error::ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> nom::IResult<&'a [u8], Self, E>
  where
    Self: Sized,
  {
    let (bytes, (field_idx_diff, access_flags)) =
      tuple((parse_uleb128_nom, parse_uleb128_nom))(bytes)?;
    Ok((
      bytes,
      Self {
        field_idx_diff,
        access_flags: AccessFlags::new_field_flag(access_flags as u16),
        field: FieldIdItem::default(),
      },
    ))
  }
}
impl Display for EncodedField {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "\tname\t: {}\n\ttype\t: {}\n\taccess\t: {}",
      self.field.name(),
      self.field.descriptor(),
      self.access_flags
    )
  }
}

pub struct EncodedMethod {
  method_idx_diff: u32,
  access_flags: AccessFlags,
  code_off: u32,
  method: MethodIdItem,
  code_item: Option<CodeItem>,
}

impl Parsable for EncodedMethod {
  fn parse<'a, E: nom::error::ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> nom::IResult<&'a [u8], Self, E>
  where
    Self: Sized,
  {
    let (bytes, (method_idx_diff, access_flags, code_off)) =
      tuple((parse_uleb128_nom, parse_uleb128_nom, parse_uleb128_nom))(bytes)?;
    Ok((
      bytes,
      Self {
        method_idx_diff,
        access_flags: AccessFlags::new_method_flag(access_flags as u16),
        code_off,
        method: MethodIdItem::default(),
        code_item: None,
      },
    ))
  }
}

impl Display for EncodedMethod {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(
      f,
      "\t\tname\t: {}\n\t\ttype\t: ({}){}\n\t\taccess\t: {}",
      self.method.name(),
      self.method.param_type(),
      self.method.return_type(),
      self.access_flags
    )?;
    if let Some(code_item) = &self.code_item {
      writeln!(f, "\n\t\tcode\t-")?;
      writeln!(f, "\t\tregisterss\t: {}", code_item.registers_size)?;
      writeln!(f, "\t\tins\t: {}", code_item.ins_size)?;
      writeln!(f, "\t\touts\t: {}", code_item.outs_size)?;
      writeln!(
        f,
        "\t\tinsns size\t: {} 16-bit code units",
        code_item.insns_size
      )?;
      for (i, ins) in code_item.insns.iter().enumerate() {
        writeln!(f, "\t\t\t{:04x}:\t{:04x}", i, ins)?;
      }
    } else {
      writeln!(f, "\n\t\tcode\t: (none)")?;
    }
    Ok(())
  }
}

pub struct CodeItem {
  registers_size: u16,
  ins_size: u16,
  outs_size: u16,
  tries_size: u16,
  debug_info_off: u32,
  insns_size: u32,
  insns: Vec<u16>,
  // tries: Option<Vec<TryItem>>,
  // handlers: Option<EncodedCatchHandlerList>,
}

impl Parsable for CodeItem {
  fn parse<'a, E: nom::error::ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> nom::IResult<&'a [u8], Self, E>
  where
    Self: Sized,
  {
    let (bytes, (registers_size, ins_size, outs_size, tries_size, debug_info_off, insns_size)) =
      tuple((le_u16, le_u16, le_u16, le_u16, le_u32, le_u32))(bytes)?;
    let (bytes, insns) = count(le_u16, insns_size as usize)(bytes)?;
    let mut m_bytes = bytes;
    if insns_size % 2 == 1 && tries_size > 0 {
      let (bytes, _) = le_u16(bytes)?;
      m_bytes = bytes;
    }
    // TODO parse instructions
    Ok((
      m_bytes,
      Self {
        registers_size,
        ins_size,
        outs_size,
        tries_size,
        debug_info_off,
        insns_size,
        insns,
      },
    ))
  }
}

impl ClassDefItem {
  pub fn new(
    class_idx: u32,
    class: TypeIdItem,
    access_flags: AccessFlags,
    superclass_idx: u32,
    superclass: Option<TypeIdItem>,
    interfaces_off: u32,
    interfaces: Option<TypeList>,
    source_file_idx: Option<u32>,
    annotations_off: u32,
    class_data_off: u32,
    static_values_off: u32,
    class_data_item: Option<ClassDataItem>,
  ) -> Self {
    Self {
      class_idx,
      class,
      access_flags,
      superclass_idx,
      superclass,
      interfaces_off,
      interfaces,
      source_file_idx,
      annotations_off,
      class_data_off,
      static_values_off,
      class_data_item,
    }
  }
}

impl Display for ClassDefItem {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "\tClass descriptor\t: {}", self.class.descriptor())?;
    writeln!(f, "\tAccess flags\t: {}", self.access_flags)?;
    let super_class = if let Some(super_class) = &self.superclass {
      super_class.descriptor()
    } else {
      "Ljava/lang/Object;"
    };
    writeln!(f, "\tSuperclass\t: {}", super_class)?;
    writeln!(f, "\tInterfaces\t-")?;
    if let Some(interfaces) = &self.interfaces {
      for (interface, idx) in interfaces.list.iter().zip(0..) {
        writeln!(f, "\t\t#{}\t: {}", idx, interface.descriptor())?;
      }
    }
    writeln!(f, "\tStatic fields\t-")?;
    if let Some(class_data_item) = &self.class_data_item {
      for (field, idx) in class_data_item.static_fields.iter().zip(0..) {
        writeln!(f, "\t\t#{}\t: (in {})", idx, self.class.descriptor())?;
        writeln!(f, "{}", field)?;
      }
    }
    writeln!(f, "\tInstance fields\t-")?;
    if let Some(class_data_item) = &self.class_data_item {
      for (field, idx) in class_data_item.instance_fields.iter().zip(0..) {
        writeln!(f, "\t\t#{}\t: (in {})", idx, self.class.descriptor())?;
        writeln!(f, "{}", field)?;
      }
    }
    writeln!(f, "\tDirect methods\t-")?;
    if let Some(class_data_item) = &self.class_data_item {
      for (method, idx) in class_data_item.direct_methods.iter().zip(0..) {
        writeln!(f, "\t\t#{}\t: (in {})", idx, self.class.descriptor())?;
        writeln!(f, "{}", method)?;
      }
    }
    writeln!(f, "\tVirtual methods\t-")?;
    if let Some(class_data_item) = &self.class_data_item {
      for (method, idx) in class_data_item.virtual_methods.iter().zip(0..) {
        writeln!(f, "\t\t#{}\t: (in {})", idx, self.class.descriptor())?;
        writeln!(f, "{}", method)?;
      }
    }
    writeln!(
      f,
      "annotations_off: {}, static_values_off: {}",
      self.annotations_off, self.static_values_off
    )?;
    Ok(())
  }
}
