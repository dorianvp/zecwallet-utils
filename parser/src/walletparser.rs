use std::{io, path::Path};

use zcash_primitives::consensus::BlockHeight;

use crate::zwl::keys::Keys;

#[derive(Debug, Clone)]
pub enum WalletKeyType {
    HdDerived = 0,
    Imported = 1,
}

#[derive(Debug, Clone)]
pub struct WalletAccount {
    pub name: String,
    pub seed: Option<Vec<u8>>,
    // pub ufvk: Option<UnifiedFullViewingKey>,
    pub birthday: BlockHeight,
    pub keys: Keys,
}

#[derive(Debug)]
pub struct Wallet {
    pub wallet_name: String,
    pub version: u64,
    pub accounts: Vec<WalletAccount>,
}

pub trait WalletParser: Send + Sized {
    fn read(filename: &Path) -> io::Result<Self>;
}
