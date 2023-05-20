use std::{
  backtrace::Backtrace,
  fmt::{Debug, Display, Formatter},
};

use nom::{
  error::{self},
  Err,
};

pub struct Error {
  kind: ErrorKind,
  backtrace: Backtrace,
}

impl Debug for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#}", self)
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    if f.alternate() {
      write!(f, "{} at\n{}", self.kind, self.backtrace)
    } else {
      write!(f, "{}", self.kind)
    }
  }
}

impl std::error::Error for Error {}

impl<E: Into<ErrorKind>> From<E> for Error {
  fn from(error: E) -> Self {
    let kind = error.into();
    Self {
      kind,
      backtrace: Backtrace::capture(),
    }
  }
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
  #[error(transparent)]
  NomError {
    kind: nom::Err<nom::error::Error<Vec<u8>>>,
  },
  #[error(transparent)]
  IoError(#[from] std::io::Error),
}

impl<'a> From<Err<error::Error<&'a [u8]>>> for ErrorKind {
  fn from(e: Err<error::Error<&'a [u8]>>) -> Self {
    ErrorKind::NomError { kind: e.to_owned() }
  }
}
