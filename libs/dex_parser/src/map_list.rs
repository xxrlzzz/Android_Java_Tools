use base::Parsable;
use nom::{
  error::ParseError,
  number::complete::{le_u16, le_u32},
  sequence::tuple,
  IResult,
};

pub struct MapList {
  size: u32,
  map_item: Vec<MapItem>,
}

pub struct MapItem {
  map_item_type: u16,
  size: u32,
  offset: u32,
}

impl Parsable for MapList {
  fn parse<'a, E: nom::error::ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> nom::IResult<&'a [u8], Self, E>
  where
    Self: Sized,
  {
    let (bytes, size) = le_u32(bytes)?;
    let (bytes, map_item) = nom::multi::count(MapItem::parse, size as usize)(bytes)?;
    Ok((bytes, Self { size, map_item }))
  }
}

impl Parsable for MapItem {
  fn parse<'a, E: ParseError<&'a [u8]>>(bytes: &'a [u8]) -> IResult<&'a [u8], Self, E> {
    let (bytes, (map_item_type, _unused, size, offset)) =
      tuple((le_u16, le_u16, le_u32, le_u32))(bytes)?;
    Ok((
      bytes,
      Self {
        map_item_type,
        size,
        offset,
      },
    ))
  }
}
