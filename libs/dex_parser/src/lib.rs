use base::{error::Error, Parsable};
use raw_dex::{DexFile, FieldIdItem, MethodIdItem, StringIdItem, TypeIdItem};

mod class_def;
mod leb128;
mod map_list;
mod raw_dex;
mod type_test;
mod utf;

pub fn parse<'a>(bytes: &'a [u8]) -> Result<DexFile, Error> {
  DexFile::parse_from_u8(bytes)
}

static mut STRING_DATA_REF: Vec<StringIdItem> = vec![];
static mut TYPE_ID_REF: Vec<TypeIdItem> = vec![];
static mut METHOD_ID_REF: Vec<MethodIdItem> = vec![];
static mut FIELD_ID_REF: Vec<FieldIdItem> = vec![];

pub fn get_string_data_ref() -> &'static Vec<StringIdItem> {
  unsafe { &STRING_DATA_REF }
}

pub fn get_str_const<'a>(index: usize) -> &'a str {
  get_string_data_ref()[index].string_data.as_str()
}

pub fn get_type_id_ref() -> &'static Vec<TypeIdItem> {
  unsafe { &TYPE_ID_REF }
}

pub fn get_type_id(index: usize) -> TypeIdItem {
  get_type_id_ref()[index].clone()
}

pub fn get_method_id_ref() -> &'static Vec<MethodIdItem> {
  unsafe { &METHOD_ID_REF }
}

pub fn get_method_id(index: usize) -> MethodIdItem {
  get_method_id_ref()[index].clone()
}

pub fn get_field_id_ref() -> &'static Vec<FieldIdItem> {
  unsafe { &FIELD_ID_REF }
}

pub fn get_field_id(index: usize) -> FieldIdItem {
  get_field_id_ref()[index].clone()
}
