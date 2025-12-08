use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};

use byteorder::{LittleEndian, ReadBytesExt};
use orchard_old::tree::MerkleHashOrchard;
use zcash_client_backend::proto::service::TreeState;
use zcash_encoding::{Optional, Vector};

use crate::{
    error::WalletError,
    zwl::{
        ZwlWallet,
        block::CompactBlockData,
        data::{WalletOptions, WalletZecPriceInfo},
        read_string, read_tree,
        wallet_txns::WalletTxns,
    },
};

pub struct WalletReader;

impl WalletReader {
    pub fn max_supported_wallet_version() -> u64 {
        25
    }

    /// Public API: what you asked for.
    pub fn read(path: impl AsRef<Path>) -> Result<ZwlWallet, WalletError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::read_from_reader(reader)
    }

    /// Internal-ish: read from any reader.
    pub fn read_from_reader<R: io::Read + ReadBytesExt>(
        mut reader: R,
    ) -> Result<ZwlWallet, WalletError> {
        let version = reader.read_u64::<LittleEndian>()?;
        if version > Self::max_supported_wallet_version() {
            return Err(WalletError::UnsupportedVersion(version));
        }

        let keys = crate::zwl::keys::Keys::read(&mut reader)?;
        let blocks = Vector::read(&mut reader, |r| CompactBlockData::read(r))?;
        let transactions = WalletTxns::read(&mut reader)?;
        let chain_name = read_string(&mut reader)?;
        let wallet_options = WalletOptions::read(&mut reader)?;
        let birthday = reader.read_u64::<LittleEndian>()?;

        let verified_tree = Optional::read(&mut reader, |r| {
            use prost::Message;
            let buf = Vector::read(r, |r| r.read_u8())?;
            TreeState::decode(&buf[..]).map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("Read Error: {}", e))
            })
        })?;

        let price_info = WalletZecPriceInfo::read(&mut reader)?;

        let orchard_witnesses = if version <= 24 {
            None
        } else {
            Optional::read(&mut reader, read_tree::<MerkleHashOrchard, _>)?
        };

        Ok(ZwlWallet {
            version,
            keys,
            blocks,
            transactions,
            chain_name,
            wallet_options,
            birthday,
            verified_tree,
            orchard_witnesses,
            price_info,
        })
    }
}
