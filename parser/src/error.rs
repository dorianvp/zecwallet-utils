use std::io;

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
