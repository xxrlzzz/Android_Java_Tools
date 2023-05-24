use std::{cmp::min, fmt::Display};

use base::{access_flag::AccessFlags, Parsable};
use nom::{
  multi::count,
  number::complete::{be_u32, be_u8, le_u16, le_u32},
  sequence::tuple,
  Slice,
};

use crate::{
  class_def::{ClassDataItem, ClassDefItem},
  get_str_const, get_type_id_ref,
  leb128::parse_uleb128,
};

#[derive(Default)]
pub struct DexFile {
  dex_header: DexHeader,
  string_ids: Vec<StringIdItem>,
  type_ids: Vec<TypeIdItem>,
  proto_ids: Vec<ProtoIdItem>,
  field_ids: Vec<FieldIdItem>,
  method_ids: Vec<MethodIdItem>,
  class_defs: Vec<ClassDefItem>,
  call_site_ids: Vec<CallSiteIdItem>,
  method_handles: Vec<MethodHandleItem>,
}

#[derive(Clone)]
pub struct StringIdItem {
  string_data_off: u32,
  string_utf16_size: u32,
  pub string_data: String,
}

#[derive(Clone, Default)]
pub struct TypeIdItem {
  pub descriptor_idx: u32,
}

#[derive(Clone, Default)]
pub struct ProtoIdItem {
  shorty_idx: u32,
  return_type_idx: u32,
  return_type: TypeIdItem,
  parameters_off: u32,
  parameters_type_list: Option<TypeList>,
}

impl ProtoIdItem {
  pub fn shorty(&self) -> &str {
    get_str_const(self.shorty_idx as usize)
  }

  pub fn return_type(&self) -> &str {
    self.return_type.descriptor()
  }
}
#[derive(Clone, Default)]
pub struct FieldIdItem {
  class_idx: u16,
  class: TypeIdItem,
  type_idx: u16,
  type_item: TypeIdItem,
  name_idx: u32,
}

impl FieldIdItem {
  pub fn name(&self) -> &str {
    get_str_const(self.name_idx as usize)
  }

  pub fn descriptor(&self) -> &str {
    self.type_item.descriptor()
  }
}

impl TypeIdItem {
  pub fn descriptor(&self) -> &str {
    get_str_const(self.descriptor_idx as usize)
  }
}

#[derive(Clone, Default)]
pub struct MethodIdItem {
  class_idx: u16,
  class: TypeIdItem,
  proto_idx: u16,
  proto: ProtoIdItem,
  name_idx: u32,
}

impl MethodIdItem {
  pub fn name(&self) -> &str {
    get_str_const(self.name_idx as usize)
  }

  pub fn param_type(&self) -> &str {
    self.proto.shorty()
  }
  pub fn return_type(&self) -> &str {
    self.proto.return_type()
  }
}

pub struct CallSiteIdItem {
  call_site_off: u32,
}
pub struct MethodHandleItem {
  method_handle_type: u16,
  field_or_method_id: u16,
}

#[derive(Default, Clone)]
pub struct TypeList {
  size: u32,
  pub list: Vec<TypeIdItem>,
}

#[derive(Default)]
pub struct DexHeader {
  pub magic: u64,
  pub checksum: u32,
  pub signature: [u8; 20],
  pub file_size: u32,
  pub header_size: u32,
  pub endian_tag: u32,
  pub link_size: u32,
  pub link_off: u32,
  pub map_off: u32,
  pub string_ids_size: u32,
  pub string_ids_off: u32,
  pub type_ids_size: u32,
  pub type_ids_off: u32,
  pub proto_ids_size: u32,
  pub proto_ids_off: u32,
  pub field_ids_size: u32,
  pub field_ids_off: u32,
  pub method_ids_size: u32,
  pub method_ids_off: u32,
  pub class_defs_size: u32,
  pub class_defs_off: u32,
  pub data_size: u32,
  pub data_off: u32,
}

const DEX_MAGIC: u32 = 0x6465780a;
const NO_INDEX: u32 = 0xffffffff;

impl Parsable for DexHeader {
  fn parse<'a, E: nom::error::ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> nom::IResult<&'a [u8], Self, E>
  where
    Self: Sized,
  {
    let (bytes, (magic, version, checksum)) = tuple((be_u32, le_u32, le_u32))(bytes)?;
    if magic != DEX_MAGIC {
      log::error!("magic is not dex {}", magic);
      return Err(nom::Err::Error(E::from_error_kind(
        bytes,
        nom::error::ErrorKind::Tag,
      )));
    }
    let version_str = format!(
      "{}{}{}",
      ((version) & 0xff) - 0x30,
      ((version >> 8) & 0xff) - 0x30,
      ((version >> 16) & 0xff) - 0x30,
    );
    log::info!("dex version {}", version_str);

    let (bytes, signature) = count(be_u8, 20)(bytes)?;
    // don't support endian swap
    let (bytes, (file_size, header_size, endian_tag, link_size, link_off, map_off)) =
      tuple((le_u32, le_u32, le_u32, le_u32, le_u32, le_u32))(bytes)?;
    let (bytes, (string_ids_size, string_ids_off)) = tuple((le_u32, le_u32))(bytes)?;
    let (bytes, (type_ids_size, type_ids_off)) = tuple((le_u32, le_u32))(bytes)?;
    let (bytes, (proto_ids_size, proto_ids_off)) = tuple((le_u32, le_u32))(bytes)?;
    let (bytes, (field_ids_size, field_ids_off)) = tuple((le_u32, le_u32))(bytes)?;
    let (bytes, (method_ids_size, method_ids_off)) = tuple((le_u32, le_u32))(bytes)?;
    let (bytes, (class_defs_size, class_defs_off)) = tuple((le_u32, le_u32))(bytes)?;
    let (bytes, (data_size, data_off)) = tuple((le_u32, le_u32))(bytes)?;
    Ok((
      bytes,
      Self {
        magic: magic as u64,
        checksum,
        signature: signature.try_into().unwrap(),
        file_size,
        header_size,
        endian_tag,
        link_size,
        link_off,
        map_off,
        string_ids_size,
        string_ids_off,
        type_ids_size,
        type_ids_off,
        proto_ids_size,
        proto_ids_off,
        field_ids_size,
        field_ids_off,
        method_ids_size,
        method_ids_off,
        class_defs_size,
        class_defs_off,
        data_size,
        data_off,
      },
    ))
  }
}

impl Parsable for DexFile {
  fn parse<'a, E: nom::error::ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> nom::IResult<&'a [u8], Self, E>
  where
    Self: Sized,
  {
    let origin_bytes = bytes;
    let (bytes, dex_header) = DexHeader::parse(bytes)?;

    let (bytes, string_ids) = count(le_u32, dex_header.string_ids_size as usize)(bytes)?;
    let mut string_id_items = Vec::with_capacity(dex_header.string_ids_size as usize);
    for string_data_off in &string_ids {
      let string_data_off = *string_data_off;
      let (string_data_len, data_offset) = parse_uleb128(&origin_bytes[string_data_off as usize..]);
      let offset_byte = origin_bytes.slice(string_data_off as usize + data_offset..);
      let (_, string_data) = count(be_u8, string_data_len as usize)(offset_byte)?;
      // Test for utf16
      // let (_, utf16_str) = parse_utf16_str(offset_byte)?;
      // let utf16_str = String::from_utf16(utf16_str.as_slice()).unwrap();
      // let utf8_str = unsafe { String::from_utf8_unchecked(string_data.to_vec()) };
      // println!("string: {} {}", utf16_str, utf8_str,);

      string_id_items.push(StringIdItem {
        string_data_off,
        string_utf16_size: string_data_len,
        string_data: unsafe { String::from_utf8_unchecked(string_data.to_vec()) },
      });
    }
    let (bytes, type_ids) = count(TypeIdItem::parse, dex_header.type_ids_size as usize)(bytes)?;

    unsafe { crate::STRING_DATA_REF = string_id_items.clone() }
    unsafe { crate::TYPE_ID_REF = type_ids.clone() }

    let (bytes, proto_ids) = count(
      tuple((le_u32, le_u32, le_u32)),
      dex_header.proto_ids_size as usize,
    )(bytes)?;
    let proto_ids: Vec<ProtoIdItem> = proto_ids
      .into_iter()
      .map(|(shorty_idx, return_type_idx, parameters_off)| {
        let parameters = if parameters_off == 0 {
          None
        } else {
          let offset_byte = origin_bytes.slice(parameters_off as usize..);
          let type_list = TypeList::parse_from_u8(offset_byte).unwrap();
          Some(type_list)
        };
        ProtoIdItem {
          shorty_idx,
          return_type_idx,
          return_type: type_ids[return_type_idx as usize].clone(),
          parameters_off,
          parameters_type_list: parameters,
        }
      })
      .collect();
    let (bytes, field_ids) = count(
      tuple((le_u16, le_u16, le_u32)),
      dex_header.field_ids_size as usize,
    )(bytes)?;
    let field_ids: Vec<FieldIdItem> = field_ids
      .into_iter()
      .map(|(class_idx, type_idx, name_idx)| FieldIdItem {
        class_idx,
        class: type_ids[class_idx as usize].clone(),
        type_idx,
        type_item: type_ids[type_idx as usize].clone(),
        name_idx,
      })
      .collect();

    let (bytes, method_ids) = count(
      tuple((le_u16, le_u16, le_u32)),
      dex_header.method_ids_size as usize,
    )(bytes)?;
    let method_ids: Vec<MethodIdItem> = method_ids
      .into_iter()
      .map(|(class_idx, proto_idx, name_idx)| MethodIdItem {
        class_idx,
        class: type_ids[class_idx as usize].clone(),
        proto_idx,
        proto: proto_ids[proto_idx as usize].clone(),
        name_idx,
      })
      .collect();
    let (bytes, class_defs) = count(
      tuple((
        le_u32, le_u32, le_u32, le_u32, le_u32, le_u32, le_u32, le_u32,
      )),
      dex_header.class_defs_size as usize,
    )(bytes)?;

    // unsafe { crate::PROTO_ID_REF = proto_ids.clone() }
    unsafe { crate::FIELD_ID_REF = field_ids.clone() }
    unsafe { crate::METHOD_ID_REF = method_ids.clone() }

    let class_defs = class_defs
      .into_iter()
      .map(
        |(
          class_idx,
          access_flags,
          superclass_idx,
          interfaces_off,
          source_file_idx,
          annotations_off,
          class_data_off,
          static_values_off,
        )| {
          let superclass = if superclass_idx == NO_INDEX {
            None
          } else {
            Some(type_ids[superclass_idx as usize].clone())
          };
          let source_file_idx = if source_file_idx == NO_INDEX {
            None
          } else {
            Some(source_file_idx)
          };
          let interfaces = if interfaces_off == 0 {
            None
          } else {
            let offset_byte = origin_bytes.slice(interfaces_off as usize..);
            let type_list = TypeList::parse_from_u8(offset_byte).unwrap();
            Some(type_list)
          };

          let class_data_item = if class_data_off == 0 {
            None
          } else {
            let offset_byte = origin_bytes.slice(class_data_off as usize..);
            let class_data_item = ClassDataItem::parse_from_u8(offset_byte, origin_bytes).unwrap();
            Some(class_data_item)
          };
          ClassDefItem::new(
            class_idx,
            type_ids[class_idx as usize].clone(),
            AccessFlags::new_class_flag(access_flags as u16),
            superclass_idx,
            superclass,
            interfaces_off,
            interfaces,
            source_file_idx,
            annotations_off,
            class_data_off,
            static_values_off,
            class_data_item,
          )
        },
      )
      .collect();
    // let (bytes, call_site_ids) = count(
    //   CallSiteIdItem::parse,
    //   dex_header.call_site_ids_size as usize,
    // )(bytes)?;
    log::info!("pass");
    Ok((
      bytes,
      Self {
        dex_header,
        string_ids: string_id_items,
        type_ids,
        proto_ids,
        field_ids,
        method_ids,
        class_defs,
        ..Default::default()
      },
    ))
  }
}

impl Parsable for CallSiteIdItem {
  fn parse<'a, E: nom::error::ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> nom::IResult<&'a [u8], Self, E>
  where
    Self: Sized,
  {
    let (bytes, call_site_off) = le_u32(bytes)?;
    Ok((bytes, Self { call_site_off }))
  }
}

impl Parsable for MethodHandleItem {
  fn parse<'a, E: nom::error::ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> nom::IResult<&'a [u8], Self, E>
  where
    Self: Sized,
  {
    let (bytes, (method_handle_type, field_or_method_id)) = tuple((le_u16, le_u16))(bytes)?;
    Ok((
      bytes,
      Self {
        method_handle_type,
        field_or_method_id,
      },
    ))
  }
}

impl Parsable for TypeIdItem {
  fn parse<'a, E: nom::error::ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> nom::IResult<&'a [u8], Self, E>
  where
    Self: Sized,
  {
    let (bytes, descriptor_idx) = le_u32(bytes)?;
    Ok((bytes, Self { descriptor_idx }))
  }
}

impl Parsable for TypeList {
  fn parse<'a, E: nom::error::ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> nom::IResult<&'a [u8], Self, E>
  where
    Self: Sized,
  {
    let (bytes, size) = le_u32(bytes)?;
    let (bytes, list) = count(le_u16, size as usize)(bytes)?;
    let list = list
      .into_iter()
      .map(|type_idx| get_type_id_ref()[type_idx as usize].clone())
      .collect();
    Ok((bytes, Self { size, list }))
  }
}

impl Display for DexFile {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "header: {}", self.dex_header)?;

    writeln!(f, "string_ids: ")?;
    for string_id in self.string_ids.iter().take(min(10, self.string_ids.len())) {
      write!(f, "{} ", string_id)?;
    }
    writeln!(f, "\ntype_ids: ")?;

    for type_id in self.type_ids.iter().take(min(10, self.type_ids.len())) {
      write!(f, "{} ", type_id)?;
    }

    writeln!(f, "\nproto_ids: ")?;
    for proto_id in self.proto_ids.iter().take(min(10, self.proto_ids.len())) {
      write!(f, "{} ", proto_id)?;
    }

    writeln!(f, "\nfield_ids: ")?;
    for field_id in self.field_ids.iter().take(min(10, self.field_ids.len())) {
      write!(f, "{} ", field_id)?;
    }

    writeln!(f, "\nmethod_ids: ")?;
    for method_id in self.method_ids.iter().take(min(10, self.method_ids.len())) {
      write!(f, "{} ", method_id)?;
    }

    writeln!(f, "\nclass_defs: ")?;
    for (class_def, idx) in self.class_defs.iter().zip(0..) {
      writeln!(f, "Class #{}: ", idx)?;
      write!(f, "{} ", class_def)?;
    }
    Ok(())
  }
}

impl Display for DexHeader {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(
      f,
      "magic: {:x}, check_sum: {}, signature: {:?}， file_size: {}, 
      header_size: 0x{:x}, endian_tag: 0x{:x}",
      self.magic, self.checksum, self.signature, self.file_size, self.header_size, self.endian_tag
    )?;
    writeln!(
      f,
      "link_size: {}, link_off: {}, map_off: {}, string_ids_size: {}, string_ids_off: {}, 
      type_ids_size: {}, type_ids_off: {}, proto_ids_size: {}, proto_ids_off: {}, field_ids_size: {}, 
      field_ids_off: {}, method_ids_size: {}, method_ids_off: {}, class_defs_size: {}, 
      class_defs_off: {}, data_size: {}, data_off: {}",
      self.link_size, self.link_off, self.map_off, self.string_ids_size, self.string_ids_off, self.type_ids_size, self.type_ids_off, self.proto_ids_size, self.proto_ids_off, self.field_ids_size, self.field_ids_off, self.method_ids_size, self.method_ids_off, self.class_defs_size, self.class_defs_off, self.data_size, self.data_off
    )?;

    Ok(())
  }
}

impl Display for StringIdItem {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(
      f,
      "off: {}, len:{}, data: {}",
      self.string_data_off, self.string_utf16_size, self.string_data
    )?;
    Ok(())
  }
}

impl Display for TypeIdItem {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "descriptor: {}", self.descriptor())?;
    Ok(())
  }
}

impl Display for ProtoIdItem {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(
      f,
      "shorty: {}, return_type: {}, parameters_off: {}",
      get_str_const(self.shorty_idx as usize),
      self.return_type.descriptor(),
      self.parameters_off
    )?;
    Ok(())
  }
}

impl Display for FieldIdItem {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(
      f,
      "class: {}, type_item: {}, name: {}",
      self.class.descriptor(),
      self.type_item.descriptor(),
      get_str_const(self.name_idx as usize)
    )?;
    Ok(())
  }
}

impl Display for MethodIdItem {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(
      f,
      "class: {}, proto: {}, name: {}",
      self.class.descriptor(),
      self.proto,
      get_str_const(self.name_idx as usize)
    )?;
    Ok(())
  }
}
