use std::fmt;

#[derive(Debug)]
pub enum Error {
    ForkFailed(nix::Error),
    AlreadyStarted,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {

}

