pub mod access_flag;
pub mod error;

pub trait RenderSource {
  fn render_file_info(&self) -> Vec<String>;
  fn render_class_info(&self) -> Vec<String>;
  fn render_interfaces(&self) -> Vec<String>;
  fn render_fields(&self) -> Vec<String>;
  fn render_methods(&self) -> Vec<String>;
  fn render_attributes(&self) -> Vec<String>;
  fn render_constant_pool(&self) -> Vec<String>;
}

pub trait Parsable {
  fn parse<'a, E: nom::error::ParseError<&'a [u8]>>(
    bytes: &'a [u8],
  ) -> nom::IResult<&'a [u8], Self, E>
  where
    Self: Sized;

  fn parse_from_u8<'a>(bytes: &'a [u8]) -> Result<Self, crate::error::Error>
  where
    Self: Sized,
  {
    Self::parse::<nom::error::Error<_>>(bytes)
      .map(|(_, v)| v)
      .map_err(|e| e.into())
  }
}
