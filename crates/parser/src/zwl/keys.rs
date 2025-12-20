pub mod orchard;
pub mod sapling;
pub mod transparent;

use byteorder::{LittleEndian, ReadBytesExt};
use sapling_crypto::zip32::ExtendedFullViewingKey;
use std::fmt::Display;
use std::io::{self, Read};
use zcash_encoding::Vector;

use crate::zwl::keys::orchard::WalletOKey;
use crate::zwl::keys::sapling::WalletZKey;
use crate::zwl::keys::transparent::WalletTKey;
#[derive(Debug, Clone)]
pub struct Keys {
    // Is the wallet encrypted? If it is, then when writing to disk, the seed is always encrypted
    // and the individual spending keys are not written
    pub encrypted: bool,

    pub enc_seed: [u8; 48], // If locked, this contains the encrypted seed
    pub nonce: Vec<u8>,     // Nonce used to encrypt the wallet.

    pub seed: [u8; 32], // Seed phrase for this wallet. If wallet is locked, this is 0

    // List of keys, actually in this wallet. This is a combination of HD keys derived from the seed,
    // viewing keys and imported spending keys.
    pub zkeys: Vec<WalletZKey>,

    // Transparent keys. If the wallet is locked, then the secret keys will be encrypted,
    // but the addresses will be present. This Vec contains both wallet and imported tkeys
    pub tkeys: Vec<WalletTKey>,

    // Unified address (Orchard) keys actually in this wallet.
    // If wallet is locked, only viewing keys are present.
    pub okeys: Vec<WalletOKey>,
}

impl Keys {
    pub fn serialized_version() -> u64 {
        22
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u64::<LittleEndian>()?;
        if version > Self::serialized_version() {
            let e = format!(
                "Don't know how to read wallet version {}. Do you have the latest version?",
                version
            );
            return Err(io::Error::new(io::ErrorKind::InvalidData, e));
        }

        // let keys = if version <= 14 {
        //     // Keys::read_old(version, &mut reader, config)
        //     todo!("Wallets with version {} are not supported yet", version)
        // } else {
        //     Keys::read(&mut reader)
        // }?;

        // Read if wallet is encrypted
        let encrypted = reader.read_u8()? > 0;

        // Read "possible" encypted seed
        let mut enc_seed = [0u8; 48];
        reader.read_exact(&mut enc_seed)?;

        // Read nounce used for encyption
        let nonce = Vector::read(&mut reader, |r| r.read_u8())?;

        // Read "possible" clear seed
        let mut seed_bytes = [0u8; 32];
        reader.read_exact(&mut seed_bytes)?;

        // TODO: read old versions of wallet file

        let okeys = if version <= 21 {
            vec![]
        } else {
            Vector::read(&mut reader, |r| WalletOKey::read(r))?
        };

        // TODO: read old versions of wallet file
        let zkeys = Vector::read(&mut reader, |r| WalletZKey::read(r))?;

        // read wallet tkeys
        let tkeys = Vector::read(&mut reader, |r| WalletTKey::read(r))?;

        Ok(Self {
            encrypted,
            enc_seed,
            nonce,
            seed: seed_bytes,
            zkeys,
            tkeys,
            okeys,
        })
    }

    pub fn get_all_extfvks(&self) -> Vec<ExtendedFullViewingKey> {
        self.zkeys.iter().map(|zk| zk.extfvk.clone()).collect()
    }

    pub fn have_sapling_spending_key(&self, extfvk: &ExtendedFullViewingKey) -> bool {
        self.zkeys
            .iter()
            .find(|zk| zk.extfvk == *extfvk)
            .map(|zk| zk.have_spending_key())
            .unwrap_or(false)
    }

    // pub fn read_old<R: Read>(
    //     version: u64,
    //     mut reader: R,
    //     config: &LightClientConfig<P>,
    // ) -> io::Result<Self> {
    //     let encrypted = if version >= 4 {
    //         reader.read_u8()? > 0
    //     } else {
    //         false
    //     };

    //     let mut enc_seed = [0u8; 48];
    //     if version >= 4 {
    //         reader.read_exact(&mut enc_seed)?;
    //     }

    //     let nonce = if version >= 4 {
    //         Vector::read(&mut reader, |r| r.read_u8())?
    //     } else {
    //         vec![]
    //     };

    //     // Seed
    //     let mut seed_bytes = [0u8; 32];
    //     reader.read_exact(&mut seed_bytes)?;

    //     let zkeys = if version <= 6 {
    //         // Up until version 6, the wallet keys were written out individually
    //         // Read the spending keys
    //         let extsks = Vector::read(&mut reader, |r| ExtendedSpendingKey::read(r))?;

    //         let extfvks = if version >= 4 {
    //             // Read the viewing keys
    //             Vector::read(&mut reader, |r| ExtendedFullViewingKey::read(r))?
    //         } else {
    //             // Calculate the viewing keys
    //             extsks
    //                 .iter()
    //                 .map(ExtendedFullViewingKey::from)
    //                 .collect::<Vec<ExtendedFullViewingKey>>()
    //         };

    //         // Calculate the addresses
    //         let addresses = extfvks
    //             .iter()
    //             .map(|fvk| fvk.default_address().1)
    //             .collect::<Vec<PaymentAddress>>();

    //         // If extsks is of len 0, then this wallet is locked
    //         let zkeys_result = if extsks.is_empty() {
    //             // Wallet is locked, so read only the viewing keys.
    //             extfvks
    //                 .iter()
    //                 .zip(addresses.iter())
    //                 .enumerate()
    //                 .map(|(i, (extfvk, payment_address))| {
    //                     let zk = WalletZKey::new_locked_hdkey(i as u32, extfvk.clone());
    //                     if zk.zaddress != *payment_address {
    //                         Err(io::Error::new(
    //                             ErrorKind::InvalidData,
    //                             "Payment address didn't match",
    //                         ))
    //                     } else {
    //                         Ok(zk)
    //                     }
    //                 })
    //                 .collect::<Vec<io::Result<WalletZKey>>>()
    //         } else {
    //             // Wallet is unlocked, read the spending keys as well
    //             extsks
    //                 .into_iter()
    //                 .zip(extfvks.into_iter().zip(addresses.iter()))
    //                 .enumerate()
    //                 .map(|(i, (extsk, (extfvk, payment_address)))| {
    //                     let zk = WalletZKey::new_hdkey(i as u32, extsk);
    //                     if zk.zaddress != *payment_address {
    //                         return Err(io::Error::new(
    //                             ErrorKind::InvalidData,
    //                             "Payment address didn't match",
    //                         ));
    //                     }

    //                     if zk.extfvk != extfvk {
    //                         return Err(io::Error::new(
    //                             ErrorKind::InvalidData,
    //                             "Full View key didn't match",
    //                         ));
    //                     }

    //                     Ok(zk)
    //                 })
    //                 .collect::<Vec<io::Result<WalletZKey>>>()
    //         };

    //         // Convert vector of results into result of vector, returning an error if any one of the keys failed the checks above
    //         zkeys_result.into_iter().collect::<io::Result<_>>()?
    //     } else {
    //         // After version 6, we read the WalletZKey structs directly
    //         Vector::read(&mut reader, |r| WalletZKey::read(r))?
    //     };

    //     let tkeys = if version <= 20 {
    //         let tkeys = Vector::read(&mut reader, |r| {
    //             let mut tpk_bytes = [0u8; 32];
    //             r.read_exact(&mut tpk_bytes)?;
    //             secp256k1::SecretKey::from_slice(&tpk_bytes)
    //                 .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
    //         })?;

    //         let taddresses = if version >= 4 {
    //             // Read the addresses
    //             Vector::read(&mut reader, |r| utils::read_string(r))?
    //         } else {
    //             // Calculate the addresses
    //             tkeys
    //                 .iter()
    //                 .map(|sk| {
    //                     WalletTKey::address_from_prefix_sk(&config.base58_pubkey_address(), sk)
    //                 })
    //                 .collect()
    //         };

    //         tkeys
    //             .iter()
    //             .zip(taddresses.iter())
    //             .enumerate()
    //             .map(|(i, (sk, taddr))| WalletTKey::from_raw(sk, taddr, i as u32))
    //             .collect::<Vec<_>>()
    //     } else {
    //         // Read the TKeys
    //         Vector::read(&mut reader, |r| WalletTKey::read(r))?
    //     };

    //     Ok(Self {
    //         config: config.clone(),
    //         encrypted,
    //         unlocked: !encrypted,
    //         enc_seed,
    //         nonce,
    //         seed: seed_bytes,
    //         zkeys,
    //         tkeys,
    //         okeys: vec![],
    //     })
    // }
}

impl Display for Keys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, ">> Keys << ").unwrap();
        writeln!(f, "Version: {}", Keys::serialized_version()).unwrap();
        writeln!(f, "Encrypted: {}", self.encrypted).unwrap();

        match self.encrypted {
            true => {
                writeln!(f, "Encrypted seed: {}", hex::encode(self.enc_seed)).unwrap();
                writeln!(f, "Nonce: {}", hex::encode(&self.nonce)).unwrap();
            }
            false => {
                writeln!(f, "Seed: {}", hex::encode(self.seed)).unwrap();
            }
        }

        writeln!(f, "=== ORCHARD ===").unwrap();
        writeln!(f, "Orchard keys found: {}", self.okeys.len()).unwrap();

        for okey in &self.okeys {
            writeln!(f, "{}", okey).unwrap();
        }

        writeln!(f, "=== SAPLING ===").unwrap();
        writeln!(f, "Sapling keys found: {}", self.zkeys.len()).unwrap();

        for zkey in &self.zkeys {
            writeln!(f, "{}", zkey).unwrap();
        }

        writeln!(f, "=== TRANSPARENT ===").unwrap();
        writeln!(f, "Transparent keys found: {}", self.tkeys.len()).unwrap();
        for tkey in &self.tkeys {
            writeln!(f, "{}", tkey).unwrap();
        }
        Ok(())
    }
}
