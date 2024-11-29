use std::fmt::{Debug, Display};
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

pub struct Error {
    kind: ErrorKind,
}

pub enum ErrorKind {
    SizeError(usize),
    AlignmentError(usize),
    IoError(io::Error),
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match &self.kind {
            ErrorKind::SizeError(size) => format!(
                "size of object must be less than or equal to {} bytes",
                size
            ),
            ErrorKind::AlignmentError(align) => {
                format!("alignment of object must have an alignment of {}", align)
            }
            ErrorKind::IoError(err) => format!("io error: {}", err),
        };
        write!(f, "{}", msg)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::new(ErrorKind::IoError(value))
    }
}

impl From<nix::Error> for Error {
    fn from(value: nix::Error) -> Self {
        Error::new(ErrorKind::IoError(value.into()))
    }
}
