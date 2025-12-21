use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};

use byteorder::{LittleEndian, ReadBytesExt};
use orchard_old::tree::MerkleHashOrchard;
use tracing::instrument;
use zcash_client_backend::proto::service::TreeState;
use zcash_encoding::{Optional, Vector};

use crate::{
    error::WalletError,
    zwl::{
        ZwlWallet,
        block::CompactBlockData,
        data::{ChainType, WalletOptions, WalletZecPriceInfo},
        read_string, read_tree,
        wallet_txns::WalletTxns,
    },
};

pub struct WalletReader;

impl WalletReader {
    pub fn max_supported_wallet_version() -> u64 {
        25
    }

    #[instrument(level = "info", name = "WalletReader::read", skip_all, fields(path = %path.as_ref().display()))]
    pub fn read(path: impl AsRef<Path>) -> Result<ZwlWallet, WalletError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::read_from_reader(reader)
    }

    #[instrument(level = "info", name = "WalletReader::read_from_reader", skip_all, err)]
    pub fn read_from_reader<R: io::Read + ReadBytesExt>(
        mut reader: R,
    ) -> Result<ZwlWallet, WalletError> {
        let version = reader.read_u64::<LittleEndian>()?;
        if version > Self::max_supported_wallet_version() {
            return Err(WalletError::UnsupportedVersion(version));
        }

        let keys = if version <= 14 {
            // Keys::read_old(version, &mut reader, config)
            todo!("Wallets with version {} are not supported yet", version)
        } else {
            crate::zwl::keys::Keys::read(&mut reader)?
        };

        let mut blocks = Vector::read(&mut reader, |r| CompactBlockData::read(r))?;
        if version <= 14 {
            // Reverse the order, since after version 20, we need highest-block-first
            blocks = blocks.into_iter().rev().collect();
        }

        let mut transactions = if version <= 14 {
            // WalletTxns::read_old(&mut reader)
            todo!(
                "Transaction parsing for wallets with version {} are not supported yet",
                version
            )
        } else {
            WalletTxns::read(&mut reader)
        }?;

        // If version <= 8, adjust the "is_spendable" status of each note data
        if version <= 8 {
            // Collect all spendable keys
            let spendable_keys: Vec<_> = keys
                .get_all_extfvks()
                .into_iter()
                .filter(|extfvk| keys.have_sapling_spending_key(extfvk))
                .collect();

            transactions.adjust_spendable_status(spendable_keys);
        }

        let chain_name = ChainType::from(read_string(&mut reader)?);
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
