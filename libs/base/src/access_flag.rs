use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub enum AccessFlag {
  Public,
  Private,
  Protected,
  Static,
  Final,
  Super,
  Synchronized,
  Volatile,
  Bridge,
  Transient,
  Varargs,
  Native,
  Interface,
  Abstract,
  Strict,
  Synthetic,
  Annotation,
  Enum,
  // Module,
}

const ACC_PUBLIC: u16 = 0x0001;
const ACC_PRIVATE: u16 = 0x0002;
const ACC_PROTECTED: u16 = 0x0004;
const ACC_STATIC: u16 = 0x0008;
const ACC_FINAL: u16 = 0x0010;
const ACC_SUPER: u16 = 0x0020;
const ACC_SYNCHRONIZED: u16 = 0x0020;
const ACC_VOLATILE: u16 = 0x0040;
const ACC_BRIDGE: u16 = 0x0040;
const ACC_TRANSIENT: u16 = 0x0080;
const ACC_VARARGS: u16 = 0x0080;
const ACC_NATIVE: u16 = 0x0100;
const ACC_INTERFACE: u16 = 0x0200;
const ACC_ABSTRACT: u16 = 0x0400;
const ACC_STRICT: u16 = 0x0800;
const ACC_SYNTHETIC: u16 = 0x1000;
const ACC_ANNOTATION: u16 = 0x2000;
const ACC_ENUM: u16 = 0x4000;

// for android
// const ACC_CONSTRUCTOR: u32 = 0x10000;
// const ACC_DECLARED_SYNCHRONIZED: u32 = 0x20000;

const CLASS_ACC: &[(u16, AccessFlag)] = &[
  (ACC_PUBLIC, AccessFlag::Public),
  (ACC_FINAL, AccessFlag::Final),
  (ACC_SUPER, AccessFlag::Super),
  (ACC_INTERFACE, AccessFlag::Interface),
  (ACC_ABSTRACT, AccessFlag::Abstract),
  (ACC_SYNTHETIC, AccessFlag::Synthetic),
  (ACC_ANNOTATION, AccessFlag::Annotation),
  (ACC_ENUM, AccessFlag::Enum),
];

const FIELD_ACC: &[(u16, AccessFlag)] = &[
  (ACC_PUBLIC, AccessFlag::Public),
  (ACC_PRIVATE, AccessFlag::Private),
  (ACC_PROTECTED, AccessFlag::Protected),
  (ACC_STATIC, AccessFlag::Static),
  (ACC_FINAL, AccessFlag::Final),
  (ACC_VOLATILE, AccessFlag::Volatile),
  (ACC_TRANSIENT, AccessFlag::Transient),
  (ACC_SYNTHETIC, AccessFlag::Synthetic),
  (ACC_ENUM, AccessFlag::Enum),
];

const METHOD_ACC: &[(u16, AccessFlag)] = &[
  (ACC_PUBLIC, AccessFlag::Public),
  (ACC_PRIVATE, AccessFlag::Private),
  (ACC_PROTECTED, AccessFlag::Protected),
  (ACC_STATIC, AccessFlag::Static),
  (ACC_FINAL, AccessFlag::Final),
  (ACC_SYNCHRONIZED, AccessFlag::Synchronized),
  (ACC_BRIDGE, AccessFlag::Bridge),
  (ACC_VARARGS, AccessFlag::Varargs),
  (ACC_NATIVE, AccessFlag::Native),
  (ACC_ABSTRACT, AccessFlag::Abstract),
  (ACC_STRICT, AccessFlag::Strict),
  (ACC_SYNTHETIC, AccessFlag::Synthetic),
];

pub struct AccessFlags(Vec<AccessFlag>, u16);

impl AccessFlags {
  pub fn new_class_flag(flag: u16) -> Self {
    let flags = CLASS_ACC
      .iter()
      .filter(|(i, _)| flag & i == *i)
      .map(|(_, af)| *af)
      .collect();
    Self(flags, flag)
  }

  pub fn new_field_flag(flag: u16) -> Self {
    let flags = FIELD_ACC
      .iter()
      .filter(|(i, _)| flag & i == *i)
      .map(|(_, af)| *af)
      .collect();
    Self(flags, flag)
  }

  pub fn new_method_flag(flag: u16) -> Self {
    let flags = METHOD_ACC
      .iter()
      .filter(|(i, _)| flag & i == *i)
      .map(|(_, af)| *af)
      .collect();
    Self(flags, flag)
  }
}

impl Display for AccessFlags {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let flags = self.0.clone();
    // flags.sort_by(|a, b| a.cmp(b));
    let mut iter = flags.iter();
    write!(f, "0x{:04x} (", self.1)?;
    if let Some(flag) = iter.next() {
      write!(f, "{}", flag)?;
      for flag in iter {
        write!(f, ",{}", flag)?;
      }
    }
    write!(f, ")")?;
    Ok(())
  }
}

impl Display for AccessFlag {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AccessFlag::Public => write!(f, "ACC_PUBLIC"),
      AccessFlag::Private => write!(f, "ACC_PRIVATE"),
      AccessFlag::Protected => write!(f, "ACC_PROTECTED"),
      AccessFlag::Static => write!(f, "ACC_STATIC"),
      AccessFlag::Final => write!(f, "ACC_FINAL"),
      AccessFlag::Super => write!(f, "ACC_SUPER"),
      AccessFlag::Synchronized => write!(f, "ACC_SYNCHRONIZED"),
      AccessFlag::Volatile => write!(f, "ACC_VOLATILE"),
      AccessFlag::Bridge => write!(f, "ACC_BRIDGE"),
      AccessFlag::Transient => write!(f, "ACC_TRANSIENT"),
      AccessFlag::Varargs => write!(f, "ACC_VARARGS"),
      AccessFlag::Native => write!(f, "ACC_NATIVE"),
      AccessFlag::Interface => write!(f, "ACC_INTERFACE"),
      AccessFlag::Abstract => write!(f, "ACC_ABSTRACE"),
      AccessFlag::Strict => write!(f, "ACC_STRICT"),
      AccessFlag::Synthetic => write!(f, "ACC_SYNTHETIC"),
      AccessFlag::Annotation => write!(f, "ACC_ANNOTATION"),
      AccessFlag::Enum => write!(f, "ACC_ENUM"),
    }?;
    Ok(())
  }
}
