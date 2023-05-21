use std::fmt::Display;

use nom::{error::ParseError, number::complete::be_u8, IResult};

use crate::Parsable;

/// https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-6.html#jvms-6.5
///
pub mod opcodes_implied {
  use std::collections::HashMap;

  pub const ACONST_NULL: u8 = 0x01;
  pub const ICONST_M1: u8 = 0x02;
  pub const ICONST_0: u8 = 0x03;
  pub const ICONST_1: u8 = 0x04;
  pub const ICONST_2: u8 = 0x05;
  pub const ICONST_3: u8 = 0x06;
  pub const ICONST_4: u8 = 0x07;
  pub const ICONST_5: u8 = 0x08;
  pub const LCONST_0: u8 = 0x09;
  pub const LCONST_1: u8 = 0x0a;
  pub const FCONST_0: u8 = 0x0b;
  pub const FCONST_1: u8 = 0x0c;
  pub const FCONST_2: u8 = 0x0d;
  pub const DCONST_0: u8 = 0x0e;
  pub const DCONST_1: u8 = 0x0f;
  pub const BIPUSH: u8 = 0x10;
  pub const SIPUSH: u8 = 0x11;
  pub const LDC: u8 = 0x12;
  pub const LDC_W: u8 = 0x13;
  pub const LDC2_W: u8 = 0x14;
  pub const DLOAD_0: u8 = 0x26;
  pub const DLOAD_1: u8 = 0x27;
  pub const DLOAD_2: u8 = 0x28;
  pub const DLOAD_3: u8 = 0x29;
  pub const ALOAD_0: u8 = 0x2a;
  pub const ALOAD_1: u8 = 0x2b;
  pub const ALOAD_2: u8 = 0x2c;
  pub const ALOAD_3: u8 = 0x2d;
  pub const AALOAD: u8 = 0x32;
  pub const AASTORE: u8 = 0x53;
  pub const DRETURN: u8 = 0xaf;
  pub const RETURN: u8 = 0xb1;
  pub const GETFIELD: u8 = 0xb4;
  pub const PUTFIELD: u8 = 0xb5;
  pub const INVOKESPACIAL: u8 = 0xb7;
  pub const ANEWARRAY: u8 = 0xbd;

  lazy_static::lazy_static! {
    pub static ref CODE_NAME_MAP: HashMap<u8, &'static str> = {
      HashMap::from([
        (AALOAD, "aaload"),
        (AASTORE, "aastore"),
        (ACONST_NULL, "aconst_null"),
        (ICONST_M1, "iconst_m1"),
        (ICONST_0, "iconst_0"),
        (ICONST_1, "iconst_1"),
        (ICONST_2, "iconst_2"),
        (ICONST_3, "iconst_3"),
        (ICONST_4, "iconst_4"),
        (ICONST_5, "iconst_5"),
        (LCONST_0, "lconst_0"),
        (LCONST_1, "lconst_1"),
        (FCONST_0, "fconst_0"),
        (FCONST_1, "fconst_1"),
        (FCONST_2, "fconst_2"),
        (DCONST_0, "dconst_0"),
        (DCONST_1, "dconst_1"),
        (BIPUSH, "bipush"),
        (SIPUSH, "sipush"),
        (LDC, "ldc"),
        (LDC_W, "ldc_w"),
        (LDC2_W, "ldc2_w"),
        (ALOAD_0, "aload_0"),
        (ALOAD_1, "aload_1"),
        (ALOAD_2, "aload_2"),
        (ALOAD_3, "aload_3"),
        (DLOAD_0, "dload_0"),
        (DLOAD_1, "dload_1"),
        (DLOAD_2, "dload_2"),
        (DLOAD_3, "dload_3"),
        (RETURN, "return"),
        (DRETURN, "dreturn"),
        (GETFIELD, "getfield"),
        (PUTFIELD, "putfield"),
        (INVOKESPACIAL, "invokespecial"),
        (ANEWARRAY, "anewarray"),
      ])
    };
    pub static ref CODE_OP_CNT_MAP: HashMap<u8, u8> = {
      HashMap::from([
        (AALOAD, 0),
        (AASTORE, 0),
        (ACONST_NULL, 0),
        (ICONST_M1, 0),
        (ICONST_0, 0),
        (ICONST_1, 0),
        (ICONST_2, 0),
        (ICONST_3, 0),
        (ICONST_4, 0),
        (ICONST_5, 0),
        (LCONST_0, 0),
        (LCONST_1, 0),
        (FCONST_0, 0),
        (FCONST_1, 0),
        (FCONST_2, 0),
        (DCONST_0, 0),
        (DCONST_1, 0),
        (BIPUSH, 1),
        (SIPUSH, 2),
        (LDC, 1),
        (LDC_W, 2),
        (LDC2_W, 2),
        (ALOAD_0, 0),
        (ALOAD_1, 0),
        (ALOAD_2, 0),
        (ALOAD_3, 0),
        (DLOAD_0, 0),
        (DLOAD_1, 0),
        (DLOAD_2, 0),
        (DLOAD_3, 0),
        (RETURN, 0),
        (DRETURN, 0),
        (GETFIELD, 2),
        (PUTFIELD, 2),
        (INVOKESPACIAL, 2),
        (ANEWARRAY, 2),
      ])
    };
  }
}

#[derive(Debug, Clone)]
pub struct CodeInfo {
  code: u8,
  index_byte1: Option<u8>,
  index_byte2: Option<u8>,
}

impl Parsable for CodeInfo {
  fn parse<'a, E: ParseError<&'a [u8]>>(bytes: &'a [u8]) -> IResult<&'a [u8], Self, E> {
    let (bytes, code) = be_u8(bytes)?;
    let (bytes, (code, index_byte1, index_byte2)) =
      match opcodes_implied::CODE_OP_CNT_MAP.get(&code) {
        Some(&0) => Ok((bytes, (code, None, None))),
        Some(&1) => {
          let (bytes, b1) = be_u8(bytes)?;
          Ok((bytes, (code, Some(b1), None)))
        }
        Some(&2) => {
          let (bytes, b1) = be_u8(bytes)?;
          let (bytes, b2) = be_u8(bytes)?;
          Ok((bytes, (code, Some(b1), Some(b2))))
        }
        None => {
          println!("unknown code: {:2x}", code);
          Ok((bytes, (code, None, None)))
        }
        _ => unreachable!(),
      }?;
    Ok((
      bytes,
      Self {
        code,
        index_byte1,
        index_byte2,
      },
    ))
  }
}

impl Display for CodeInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let code_name = opcodes_implied::CODE_NAME_MAP.get(&self.code).unwrap();
    if let Some(b1) = self.index_byte1 {
      if let Some(b2) = self.index_byte2 {
        write!(f, "{}<{} {}>", code_name, b1, b2)
      } else {
        write!(f, "{}<{}>", code_name, b1)
      }
    } else {
      write!(f, "{}", code_name)
    }
  }
}
