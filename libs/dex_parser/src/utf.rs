use nom::number::complete::be_u8;

fn parse_utf16<'a, E: nom::error::ParseError<&'a [u8]>>(
  bytes: &'a [u8],
) -> nom::IResult<&'a [u8], u32, E> {
  let (bytes, one) = be_u8(bytes)?;
  if one & 0x80 == 0 {
    return Ok((bytes, one as u32));
  }
  let (bytes, two) = be_u8(bytes)?;
  if one & 0x20 == 0 {
    return Ok((bytes, ((one & 0x1f) as u32) << 6 | ((two & 0x3f) as u32)));
  }
  let (bytes, three) = be_u8(bytes)?;
  if one & 0x10 == 0 {
    return Ok((
      bytes,
      ((one & 0x0f) as u32) << 12 | ((two & 0x3f) as u32) << 6 | ((three & 0x3f) as u32),
    ));
  }
  let (bytes, four) = be_u8(bytes)?;
  let code_point = ((one & 0x07) as u32) << 18
    | ((two & 0x3f) as u32) << 12
    | ((three & 0x3f) as u32) << 6
    | ((four & 0x3f) as u32);
  let surrogate_pair =
    (((code_point >> 10) + 0xd7c0) & 0xffff) | (((code_point & 0x03ff) + 0xdc00) << 16);
  Ok((bytes, surrogate_pair))
}

pub fn parse_utf16_str<'a, E: nom::error::ParseError<&'a [u8]>>(
  bytes: &'a [u8],
) -> nom::IResult<&'a [u8], Vec<u16>, E> {
  let mut m_bytes = bytes;
  let mut res = vec![];
  while m_bytes[0] != 0 {
    let (bytes, ch) = parse_utf16::<E>(m_bytes)?;
    let (leading, tailing) = (ch & 0xffff, ch >> 16);
    res.push(leading as u16);
    if tailing != 0 {
      res.push(tailing as u16);
    }
    m_bytes = bytes;
  }
  Ok((m_bytes, res))
}
