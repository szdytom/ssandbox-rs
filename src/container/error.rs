use std::fmt;

#[derive(Debug)]
pub enum Error {
    ForkFailed(nix::Error),
    AlreadyStarted,
    EntryError(EntryError),
}

#[derive(Debug)]
pub struct EntryError {
    error_code: u8,
    additional_info: String,
}

impl EntryError {
    pub fn new(code: u8, buf: &[u8]) -> Self {
        Self {
            error_code: code,
            additional_info: String::from_utf8_lossy(buf).into_owned(),
        }
    }
}

impl std::convert::Into<Error> for EntryError {
    fn into(self) -> Error {
        Error::EntryError(self)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
