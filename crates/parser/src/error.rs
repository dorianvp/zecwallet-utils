use std::{fmt, io};

#[derive(Debug)]
pub enum WalletError {
    Io(io::Error),
    UnsupportedVersion(u64),
    InvalidFormat(String),
}

impl From<io::Error> for WalletError {
    fn from(e: io::Error) -> Self {
        WalletError::Io(e)
    }
}

impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WalletError::Io(e) => write!(f, "IO error: {}", e),
            WalletError::UnsupportedVersion(v) => write!(f, "Unsupported wallet version: {}", v),
            WalletError::InvalidFormat(s) => write!(f, "Invalid wallet format: {}", s),
        }
    }
}
