use std::fmt::Display;

use nom::number::complete::{be_u16, be_u8};

use crate::Parsable;

#[derive(Clone)]
pub struct StackMapTable {
  number_of_entries: u16,
  entries: Vec<StackMapFrame>,
}

#[derive(Clone)]
pub enum StackMapFrame {
  SameFrame(u8),
  SameLocals1StackItemFrame((u8, VerificationTypeInfo)),
  SameLocals1StackItemFrameExtended((u8, u16, VerificationTypeInfo)),
  ChopFrame((u8, u16)),
  SameFrameExtended((u8, u16)),
  AppendFrame((u8, u16, Vec<VerificationTypeInfo>)),
  FullFrame(
    (
      u8,
      u16,
      Vec<VerificationTypeInfo>,
      Vec<VerificationTypeInfo>,
    ),
  ),
  Invalid,
}

#[derive(Clone)]
pub enum VerificationTypeInfo {
  Top,
  Integer,
  Float,
  Long,
  Double,
  Null,
  UninitializedThis,
  Object(u16),
  Uninitialized(u16),
  Invalid,
}

impl Parsable for StackMapTable {
  fn parse<'a, E: nom::error::ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> nom::IResult<&'a [u8], Self, E>
  where
    Self: Sized,
  {
    let (bytes, number_of_entries) = nom::number::complete::be_u16(bytes)?;
    let (bytes, entries) =
      nom::multi::count(StackMapFrame::parse, number_of_entries as usize)(bytes)?;
    Ok((
      bytes,
      Self {
        number_of_entries,
        entries,
      },
    ))
  }
}

impl Parsable for StackMapFrame {
  fn parse<'a, E: nom::error::ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> nom::IResult<&'a [u8], Self, E>
  where
    Self: Sized,
  {
    let (bytes, frame_type) = be_u8(bytes)?;
    let mut m_bytes = bytes;
    let frame = match frame_type {
      0..=63 => Self::SameFrame(frame_type),
      64..=127 => {
        let (bytes, verification_type_info) = VerificationTypeInfo::parse(bytes)?;
        m_bytes = bytes;
        Self::SameLocals1StackItemFrame((frame_type, verification_type_info))
      }
      128..=246 => Self::Invalid,
      247 => {
        let (bytes, offset_delta) = be_u16(bytes)?;
        let (bytes, verification_type_info) = VerificationTypeInfo::parse(bytes)?;
        m_bytes = bytes;
        Self::SameLocals1StackItemFrameExtended((frame_type, offset_delta, verification_type_info))
      }
      248..=250 => {
        let (bytes, offset_delta) = be_u16(bytes)?;
        m_bytes = bytes;
        Self::ChopFrame((frame_type, offset_delta))
      }
      251 => {
        let (bytes, offset_delta) = be_u16(bytes)?;
        m_bytes = bytes;
        Self::SameFrameExtended((frame_type, offset_delta))
      }
      252..=254 => {
        let (bytes, offset_delta) = be_u16(bytes)?;
        let (bytes, verification_type_info) =
          nom::multi::count(VerificationTypeInfo::parse, (frame_type - 251) as usize)(bytes)?;
        m_bytes = bytes;
        Self::AppendFrame((frame_type, offset_delta, verification_type_info))
      }
      255 => {
        let (bytes, offset_delta) = be_u16(bytes)?;
        let (bytes, number_of_locals) = be_u16(bytes)?;
        let (bytes, locals) =
          nom::multi::count(VerificationTypeInfo::parse, number_of_locals as usize)(bytes)?;
        let (bytes, number_of_stack_items) = be_u16(bytes)?;
        let (bytes, stack) =
          nom::multi::count(VerificationTypeInfo::parse, number_of_stack_items as usize)(bytes)?;
        m_bytes = bytes;
        Self::FullFrame((frame_type, offset_delta, locals, stack))
      }
    };
    Ok((m_bytes, frame))
  }
}

impl Parsable for VerificationTypeInfo {
  fn parse<'a, E: nom::error::ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> nom::IResult<&'a [u8], Self, E>
  where
    Self: Sized,
  {
    let (bytes, tag) = be_u8(bytes)?;
    let mut m_bytes = bytes;
    let type_info = match tag {
      0 => Self::Top,
      1 => Self::Integer,
      2 => Self::Float,
      3 => Self::Long,
      4 => Self::Double,
      5 => Self::Null,
      6 => Self::UninitializedThis,
      7 => {
        let (bytes, cpool_index) = be_u16(bytes)?;
        m_bytes = bytes;
        Self::Object(cpool_index)
      }
      8 => {
        let (bytes, offset) = be_u16(bytes)?;
        m_bytes = bytes;
        Self::Uninitialized(offset)
      }
      _ => Self::Invalid,
    };
    Ok((m_bytes, type_info))
  }
}

impl Display for StackMapTable {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "StackMapTable({})", self.number_of_entries)
  }
}
