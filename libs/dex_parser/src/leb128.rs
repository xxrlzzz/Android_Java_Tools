use nom::{error::ParseError, IResult};

pub fn parse_uleb128<'a>(bytes: &'a [u8]) -> (u32, usize) {
  let mut result = 0;
  let mut shift = 0;
  let mut i = 1;
  for byte in bytes {
    result |= ((byte & 0x7f) as u32) << shift;
    if byte & 0x80 == 0 {
      break;
    }
    i += 1;
    shift += 7;
  }
  (result, i)
}

pub fn parse_uleb128_nom<'a, E: ParseError<&'a [u8]>>(bytes: &'a [u8]) -> IResult<&[u8], u32, E> {
  let (result, i) = parse_uleb128(bytes);
  Ok((bytes.split_at(i).1, result))
}
