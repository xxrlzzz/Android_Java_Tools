use std::fmt::Display;

use nom::{
  bytes::complete::take,
  combinator::map,
  error::ParseError,
  number::complete::{be_u16, be_u32, be_u8},
  IResult,
};
#[derive(Clone)]
pub struct ConstantPoolInfo {
  tag: u8,
  info: ConstantType,
}

#[derive(Clone)]
pub enum ConstantType {
  Utf8(String),
  Integer(u32),
  Float(f32),
  Long(u64),
  Double(f64),
  Class(u16),
  String(u16),
  Fieldref(u16, u16),
  Methodref(u16, u16),
  InterfaceMethodref(u16, u16),
  NameAndType(u16, u16),
  MethodHandle(u8, u16),
  MethodType(u16),
  // Dynamic,
  InvokeDynamic(u16, u16),
  // Module,
  // Package,
  Empty,
}

fn parse_float(value: u32) -> f32 {
  if value == 0x7f800000 {
    f32::INFINITY
  } else if value == 0xff800000 {
    f32::NEG_INFINITY
  } else if value == 0x7fc00000 {
    f32::NAN
  } else {
    let s = if value >> 31 == 0 { 1.0 } else { -1.0 };
    let e = ((value >> 23) & 0xff) as i32;
    let m = if e == 0 {
      (value & 0x7fffff) << 1
    } else {
      (value & 0x7fffff) | 0x800000
    };
    s * (m as f32) * 2.0f32.powi(e - 150)
  }
}

fn parse_double(value: u64) -> f64 {
  if value == 0x7ff0000000000000 {
    f64::INFINITY
  } else if value == 0xfff0000000000000 {
    f64::NEG_INFINITY
  } else if (value >= 0x7ff0000000000001 && value <= 0x7fffffffffffffff)
    || (value >= 0xfff0000000000001)
  {
    f64::NAN
  } else {
    let s = if value >> 63 == 0 { 1.0 } else { -1.0 };
    let e = ((value >> 52) & 0x7ff) as i32;
    let m = if e == 0 {
      (value & 0xfffffffffffff) << 1
    } else {
      (value & 0xfffffffffffff) | 0x10000000000000
    };
    s * (m as f64) * 2.0f64.powi(e - 1075)
  }
}

impl ConstantPoolInfo {
  pub fn parse<'a, E: ParseError<&'a [u8]>>(bytes: &'a [u8]) -> IResult<&'a [u8], Self, E> {
    let (bytes, cp) = ConstantType::parse(bytes)?;

    Ok((
      bytes,
      Self {
        tag: cp.value(),
        info: cp,
      },
    ))
  }

  pub fn is_double_size(&self) -> bool {
    match self.info {
      ConstantType::Double(_) => true,
      ConstantType::Long(_) => true,
      _ => false,
    }
  }

  pub fn is_utf8(&self) -> bool {
    match self.info {
      ConstantType::Utf8(_) => true,
      _ => false,
    }
  }

  pub fn as_utf8(&self) -> Option<&str> {
    match self.info {
      ConstantType::Utf8(ref value) => Some(value),
      _ => None,
    }
  }

  pub fn new_empty() -> Self {
    Self {
      tag: 0,
      info: ConstantType::Empty,
    }
  }
}

impl ConstantType {
  pub fn parse<'a, E: ParseError<&'a [u8]>>(bytes: &'a [u8]) -> IResult<&'a [u8], Self, E> {
    let (bytes, tag) = be_u8(bytes)?;

    match tag {
      1 => {
        let (bytes, length) = be_u16(bytes)?;
        // TODO parse utf8
        let (bytes, value) = map(take(length), |bytes: &[u8]| {
          String::from_utf8(bytes.to_vec()).unwrap()
        })(bytes)?;
        Ok((bytes, ConstantType::Utf8(value)))
      }
      3 => {
        // Integer
        // big-endian
        let (bytes, value) = be_u32(bytes)?;
        Ok((bytes, ConstantType::Integer(value)))
      }
      4 => {
        // Float
        let (bytes, value) = be_u32(bytes)?;
        Ok((bytes, ConstantType::Float(parse_float(value))))
      }
      5 => {
        let (bytes, hight_value) = be_u32(bytes)?;
        let (bytes, low_value) = be_u32(bytes)?;
        let val = (hight_value as u64) << 32 | low_value as u64;
        Ok((bytes, ConstantType::Long(val)))
      }
      6 => {
        let (bytes, hight_value) = be_u32(bytes)?;
        let (bytes, low_value) = be_u32(bytes)?;
        let val = (hight_value as u64) << 32 | low_value as u64;
        Ok((bytes, ConstantType::Double(parse_double(val))))
      }
      7 => {
        let (bytes, name_index) = be_u16(bytes)?;
        Ok((bytes, ConstantType::Class(name_index)))
      }
      8 => {
        let (bytes, string_index) = be_u16(bytes)?;
        Ok((bytes, ConstantType::String(string_index)))
      }
      9 | 10 | 11 => {
        let (bytes, class_index) = be_u16(bytes)?;
        let (bytes, name_and_type_index) = be_u16(bytes)?;
        match tag {
          9 => Ok((
            bytes,
            ConstantType::Fieldref(class_index, name_and_type_index),
          )),
          10 => Ok((
            bytes,
            ConstantType::Methodref(class_index, name_and_type_index),
          )),
          11 => Ok((
            bytes,
            ConstantType::InterfaceMethodref(class_index, name_and_type_index),
          )),
          _ => unreachable!(),
        }
      }
      12 => {
        let (bytes, name_index) = be_u16(bytes)?;
        let (bytes, descriptor_index) = be_u16(bytes)?;
        Ok((
          bytes,
          ConstantType::NameAndType(name_index, descriptor_index),
        ))
      }
      15 => {
        let (bytes, reference_kind) = be_u8(bytes)?;
        let (bytes, reference_index) = be_u16(bytes)?;
        Ok((
          bytes,
          ConstantType::MethodHandle(reference_kind, reference_index),
        ))
      }
      16 => {
        let (bytes, descriptor_index) = be_u16(bytes)?;
        Ok((bytes, ConstantType::MethodType(descriptor_index)))
      }
      // 17 => Ok((bytes, ConstantType::Dynamic)),
      18 => {
        let (bytes, bootstrap_method_attr_index) = be_u16(bytes)?;
        let (bytes, name_and_type_index) = be_u16(bytes)?;
        Ok((
          bytes,
          ConstantType::InvokeDynamic(bootstrap_method_attr_index, name_and_type_index),
        ))
      }
      // 19 => Ok((bytes, ConstantType::Module)),
      // 20 => Ok((bytes, ConstantType::Package)),
      _ => {
        println!("unknown tag: {}", tag);
        // Err(nom::Err::Error(E::from_error_kind(
        //   bytes,
        //   nom::error::ErrorKind::Tag,
        // )))
        Ok((bytes, ConstantType::Utf8("".to_string())))
      }
    }
  }

  pub fn value(&self) -> u8 {
    match self {
      ConstantType::Utf8(_) => 1,
      ConstantType::Integer(_) => 3,
      ConstantType::Float(_) => 4,
      ConstantType::Long(_) => 5,
      ConstantType::Double(_) => 6,
      ConstantType::Class(_) => 7,
      ConstantType::String(_) => 8,
      ConstantType::Fieldref(_, _) => 9,
      ConstantType::Methodref(_, _) => 10,
      ConstantType::InterfaceMethodref(_, _) => 11,
      ConstantType::NameAndType(_, _) => 12,
      ConstantType::MethodHandle(_, _) => 15,
      ConstantType::MethodType(_) => 16,
      ConstantType::InvokeDynamic(_, _) => 18,
      ConstantType::Empty => 0,
    }
  }
}

impl Display for ConstantType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ConstantType::Utf8(s) => write!(f, "Utf8: {}", s),
      ConstantType::Integer(v) => write!(f, "Integer: {}", v),
      ConstantType::Float(v) => write!(f, "Float: {}", v),
      ConstantType::Long(v) => write!(f, "Long: {}", v),
      ConstantType::Double(v) => write!(f, "Double: {}", v),
      ConstantType::Class(name_index) => write!(f, "Class: {}", name_index),
      ConstantType::String(string) => write!(f, "String: {}", string),
      ConstantType::Fieldref(class, name_and_type) => write!(
        f,
        "Fieldref: class: {}, name_and_type: {}",
        class, name_and_type
      ),
      ConstantType::Methodref(class, name_and_type) => write!(
        f,
        "Methodref: class: {}, name_and_type: {}",
        class, name_and_type
      ),
      ConstantType::InterfaceMethodref(class, name_and_type) => write!(
        f,
        "InterfaceMethodref: class: {}, name_and_type: {}",
        class, name_and_type
      ),
      ConstantType::NameAndType(name, descriptor) => {
        write!(f, "NameAndType: name: {}, descriptor: {}", name, descriptor)
      }
      ConstantType::MethodHandle(reference_kind, reference_index) => {
        write!(
          f,
          "MethodHandle: reference_kind: {}, reference_index: {}",
          reference_kind, reference_index
        )
      }
      ConstantType::MethodType(descriptor) => {
        write!(f, "MethodType: {}", descriptor)
      }
      ConstantType::InvokeDynamic(bootstrap_method_attr, name_and_type) => {
        write!(
          f,
          "InvokeDynamic: bootstrap_method_attr: {}, name_and_type: {}",
          bootstrap_method_attr, name_and_type
        )
      }
      ConstantType::Empty => write!(f, "__placeholder__"),
    }?;
    Ok(())
  }
}

impl Display for ConstantPoolInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.info)?;
    Ok(())
  }
}
