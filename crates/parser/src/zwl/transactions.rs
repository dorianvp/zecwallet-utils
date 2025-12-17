use std::io::{self};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use sapling_crypto::IncrementalWitness;
use zcash_encoding::{Optional, Vector};
use zcash_primitives::{
    consensus::BlockHeight,
    memo::{Memo, MemoBytes},
    transaction::{
        TxId,
        components::OutPoint,
    },
};

use super::{orchard_data::OrchardNoteData, sapling_data::SaplingNoteData};

pub const MAX_REORG: usize = 100;

#[derive(Clone, Debug)]
pub struct WalletTx {
    // Block in which this tx was included
    pub block: BlockHeight,

    // Is this Tx unconfirmed (i.e., not yet mined)
    pub unconfirmed: bool,

    // Timestamp of Tx. Added in v4
    pub datetime: u64,

    // Txid of this transaction. It's duplicated here (It is also the Key in the HashMap that points to this
    // WalletTx in LightWallet::txs)
    pub txid: TxId,

    // List of all nullifiers spent in this Tx. These nullifiers belong to the wallet.
    pub s_spent_nullifiers: Vec<sapling_crypto::Nullifier>,

    // List of all orchard nullifiers spent in this Tx.
    pub o_spent_nullifiers: Vec<orchard_old::note::Nullifier>,

    // List of all notes received in this tx. Some of these might be change notes.
    pub sapling_notes: Vec<SaplingNoteData>,

    // List of all orchard notes recieved in this tx. Some of these might be change.
    pub orchard_notes: Vec<OrchardNoteData>,

    // List of all Utxos received in this Tx. Some of these might be change notes
    pub utxos: Vec<Utxo>,

    // Total value of all orchard nullifiers that were spent in this Tx
    pub total_orchard_value_spent: u64,

    // Total value of all the sapling nullifiers that were spent in this Tx
    pub total_sapling_value_spent: u64,

    // Total amount of transparent funds that belong to us that were spent in this Tx.
    pub total_transparent_value_spent: u64,

    // All outgoing sapling sends to addresses outside this wallet
    pub outgoing_metadata: Vec<OutgoingTxMetadata>,

    // Whether this TxID was downloaded from the server and scanned for Memos
    pub full_tx_scanned: bool,

    // Price of Zec when this Tx was created
    pub zec_price: Option<f64>,
}

#[allow(dead_code)]
impl WalletTx {
    pub fn serialized_version() -> u64 {
        23
    }

    pub fn new_txid(txid: &Vec<u8>) -> TxId {
        let mut txid_bytes = [0u8; 32];
        txid_bytes.copy_from_slice(txid);
        TxId::from_bytes(txid_bytes)
    }

    pub fn new(height: BlockHeight, datetime: u64, txid: &TxId, unconfirmed: bool) -> Self {
        WalletTx {
            block: height,
            unconfirmed,
            datetime,
            txid: *txid,
            o_spent_nullifiers: vec![],
            s_spent_nullifiers: vec![],
            sapling_notes: vec![],
            orchard_notes: vec![],
            utxos: vec![],
            total_transparent_value_spent: 0,
            total_sapling_value_spent: 0,
            total_orchard_value_spent: 0,
            outgoing_metadata: vec![],
            full_tx_scanned: false,
            zec_price: None,
        }
    }

    pub fn read<R: ReadBytesExt>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u64::<LittleEndian>()?;

        let block = BlockHeight::from_u32(reader.read_i32::<LittleEndian>()? as u32);

        let unconfirmed = if version <= 20 {
            false
        } else {
            reader.read_u8()? == 1
        };

        let datetime = if version >= 4 {
            reader.read_u64::<LittleEndian>()?
        } else {
            0
        };

        let mut txid_bytes = [0u8; 32];
        reader.read_exact(&mut txid_bytes)?;

        let txid = TxId::from_bytes(txid_bytes);

        let s_notes = Vector::read(&mut reader, |r| SaplingNoteData::read(r))?;
        let utxos = Vector::read(&mut reader, |r| Utxo::read(r))?;

        let total_orchard_value_spent = if version <= 22 {
            0
        } else {
            reader.read_u64::<LittleEndian>()?
        };
        let total_sapling_value_spent = reader.read_u64::<LittleEndian>()?;
        let total_transparent_value_spent = reader.read_u64::<LittleEndian>()?;

        // Outgoing metadata was only added in version 2
        let outgoing_metadata = Vector::read(&mut reader, |r| OutgoingTxMetadata::read(r))?;

        let full_tx_scanned = reader.read_u8()? > 0;

        let zec_price = if version <= 4 {
            None
        } else {
            Optional::read(&mut reader, |r| r.read_f64::<LittleEndian>())?
        };

        let s_spent_nullifiers = if version <= 5 {
            vec![]
        } else {
            Vector::read(&mut reader, |r| {
                let mut n = [0u8; 32];
                r.read_exact(&mut n)?;
                Ok(sapling_crypto::Nullifier(n))
            })?
        };

        let o_notes = if version <= 21 {
            vec![]
        } else {
            Vector::read(&mut reader, |r| OrchardNoteData::read(r))?
        };

        let o_spent_nullifiers = if version <= 21 {
            vec![]
        } else {
            Vector::read(&mut reader, |r| {
                let mut rho_bytes = [0u8; 32];
                r.read_exact(&mut rho_bytes)?;
                Ok(orchard_old::note::Nullifier::from_bytes(&rho_bytes).unwrap())
            })?
        };

        Ok(Self {
            block,
            unconfirmed,
            datetime,
            txid,
            sapling_notes: s_notes,
            orchard_notes: o_notes,
            utxos,
            s_spent_nullifiers,
            o_spent_nullifiers,
            total_sapling_value_spent,
            total_orchard_value_spent,
            total_transparent_value_spent,
            outgoing_metadata,
            full_tx_scanned,
            zec_price,
        })
    }

    pub fn write<W: WriteBytesExt>(&self, mut writer: W) -> io::Result<()> {
        writer.write_u64::<LittleEndian>(Self::serialized_version())?;

        let block: u32 = self.block.into();
        writer.write_i32::<LittleEndian>(block as i32)?;

        writer.write_u8(if self.unconfirmed { 1 } else { 0 })?;

        writer.write_u64::<LittleEndian>(self.datetime)?;

        writer.write_all(self.txid.as_ref())?;

        Vector::write(&mut writer, &self.sapling_notes, |w, nd| nd.write(w))?;
        Vector::write(&mut writer, &self.utxos, |w, u| u.write(w))?;

        writer.write_u64::<LittleEndian>(self.total_orchard_value_spent)?;
        writer.write_u64::<LittleEndian>(self.total_sapling_value_spent)?;
        writer.write_u64::<LittleEndian>(self.total_transparent_value_spent)?;

        // Write the outgoing metadata
        Vector::write(&mut writer, &self.outgoing_metadata, |w, om| om.write(w))?;

        writer.write_u8(if self.full_tx_scanned { 1 } else { 0 })?;

        Optional::write(&mut writer, self.zec_price, |w, p| {
            w.write_f64::<LittleEndian>(p)
        })?;

        Vector::write(&mut writer, &self.s_spent_nullifiers, |w, n| {
            w.write_all(&n.0)
        })?;

        Vector::write(&mut writer, &self.orchard_notes, |w, n| n.write(w))?;

        Vector::write(&mut writer, &self.o_spent_nullifiers, |w, n| {
            w.write_all(&n.to_bytes())
        })?;

        Ok(())
    }

    pub fn total_funds_spent(&self) -> u64 {
        self.total_orchard_value_spent
            + self.total_sapling_value_spent
            + self.total_transparent_value_spent
    }
}

#[derive(Clone, Debug)]
pub struct Utxo {
    pub address: String,
    pub txid: TxId,
    pub output_index: u64,
    pub script: Vec<u8>,
    pub value: u64,
    pub height: i32,

    pub spent_at_height: Option<i32>,
    pub spent: Option<TxId>, // If this utxo was confirmed spent

    // If this utxo was spent in a send, but has not yet been confirmed.
    // Contains the txid and height at which the Tx was broadcast
    pub unconfirmed_spent: Option<(TxId, u32)>,
}

#[allow(dead_code)]
impl Utxo {
    pub fn serialized_version() -> u64 {
        3
    }

    pub fn to_outpoint(&self) -> OutPoint {
        OutPoint::new(*self.txid.as_ref(), self.output_index as u32)
    }

    pub fn read<R: ReadBytesExt>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u64::<LittleEndian>()?;

        let address_len = reader.read_i32::<LittleEndian>()?;
        let mut address_bytes = vec![0; address_len as usize];
        reader.read_exact(&mut address_bytes)?;
        let address = String::from_utf8(address_bytes).unwrap();
        assert_eq!(address.chars().take(1).collect::<Vec<char>>()[0], 't');

        let mut txid_bytes = [0; 32];
        reader.read_exact(&mut txid_bytes)?;
        let txid = TxId::from_bytes(txid_bytes);

        let output_index = reader.read_u64::<LittleEndian>()?;
        let value = reader.read_u64::<LittleEndian>()?;
        let height = reader.read_i32::<LittleEndian>()?;

        let script = Vector::read(&mut reader, |r| {
            let mut byte = [0; 1];
            r.read_exact(&mut byte)?;
            Ok(byte[0])
        })?;

        let spent = Optional::read(&mut reader, |r| {
            let mut txbytes = [0u8; 32];
            r.read_exact(&mut txbytes)?;
            Ok(TxId::from_bytes(txbytes))
        })?;

        let spent_at_height = if version <= 1 {
            None
        } else {
            Optional::read(&mut reader, |r| r.read_i32::<LittleEndian>())?
        };

        let unconfirmed_spent = if version <= 2 {
            None
        } else {
            Optional::read(&mut reader, |r| {
                let mut txbytes = [0u8; 32];
                r.read_exact(&mut txbytes)?;

                let height = r.read_u32::<LittleEndian>()?;
                Ok((TxId::from_bytes(txbytes), height))
            })?
        };

        Ok(Utxo {
            address,
            txid,
            output_index,
            script,
            value,
            height,
            spent_at_height,
            spent,
            unconfirmed_spent,
        })
    }

    pub fn write<W: WriteBytesExt>(&self, mut writer: W) -> io::Result<()> {
        writer.write_u64::<LittleEndian>(Self::serialized_version())?;

        writer.write_u32::<LittleEndian>(self.address.len() as u32)?;
        writer.write_all(self.address.as_bytes())?;

        writer.write_all(self.txid.as_ref())?;

        writer.write_u64::<LittleEndian>(self.output_index)?;
        writer.write_u64::<LittleEndian>(self.value)?;
        writer.write_i32::<LittleEndian>(self.height)?;

        Vector::write(&mut writer, &self.script, |w, b| w.write_all(&[*b]))?;

        Optional::write(&mut writer, self.spent, |w, txid| {
            w.write_all(txid.as_ref())
        })?;

        Optional::write(&mut writer, self.spent_at_height, |w, s| {
            w.write_i32::<LittleEndian>(s)
        })?;

        Optional::write(&mut writer, self.unconfirmed_spent, |w, (txid, height)| {
            w.write_all(txid.as_ref())?;
            w.write_u32::<LittleEndian>(height)
        })?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct WitnessCache {
    pub witnesses: Vec<IncrementalWitness>,
    pub top_height: u64,
}

#[allow(dead_code)]
impl WitnessCache {
    pub fn new(witnesses: Vec<IncrementalWitness>, top_height: u64) -> Self {
        Self {
            witnesses,
            top_height,
        }
    }

    pub fn empty() -> Self {
        Self {
            witnesses: vec![],
            top_height: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.witnesses.len()
    }

    pub fn is_empty(&self) -> bool {
        self.witnesses.is_empty()
    }

    pub fn clear(&mut self) {
        self.witnesses.clear();
    }

    pub fn get(&self, i: usize) -> Option<&IncrementalWitness> {
        self.witnesses.get(i)
    }

    #[cfg(test)]
    pub fn get_from_last(&self, i: usize) -> Option<&IncrementalWitness> {
        self.witnesses.get(self.len() - i - 1)
    }

    pub fn last(&self) -> Option<&IncrementalWitness> {
        self.witnesses.last()
    }

    // pub fn into_fsb(self, fsb: &mut FixedSizeBuffer<IncrementalWitness>) {
    //     self.witnesses.into_iter().for_each(|w| fsb.push(w));
    // }

    pub fn pop(&mut self, at_height: u64) {
        while !self.witnesses.is_empty() && self.top_height >= at_height {
            self.witnesses.pop();
            self.top_height -= 1;
        }
    }

    // pub fn get_as_string(&self, i: usize) -> String {
    //     if i >= self.witnesses.len() {
    //         return "".to_string();
    //     }

    //     let mut buf = vec![];
    //     self.get(i).unwrap().write(&mut buf).unwrap();
    //     return hex::encode(buf);
    // }
}

#[derive(PartialEq, Clone, Debug)]
pub struct OutgoingTxMetadata {
    pub address: String,
    pub value: u64,
    pub memo: Memo,
}

impl OutgoingTxMetadata {
    pub fn read<R: ReadBytesExt>(mut reader: R) -> io::Result<Self> {
        let address_len = reader.read_u64::<LittleEndian>()?;
        let mut address_bytes = vec![0; address_len as usize];
        reader.read_exact(&mut address_bytes)?;
        let address = String::from_utf8(address_bytes).unwrap();

        let value = reader.read_u64::<LittleEndian>()?;

        let mut memo_bytes = [0u8; 512];
        reader.read_exact(&mut memo_bytes)?;
        let memo = match MemoBytes::from_bytes(&memo_bytes) {
            Ok(mb) => match Memo::try_from(mb.clone()) {
                Ok(m) => Ok(m),
                Err(_) => Ok(Memo::Future(mb)),
            },
            Err(e) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Couldn't create memo: {}", e),
            )),
        }?;

        Ok(OutgoingTxMetadata {
            address,
            value,
            memo,
        })
    }

    pub fn write<W: WriteBytesExt>(&self, mut writer: W) -> io::Result<()> {
        // Strings are written as len + utf8
        writer.write_u64::<LittleEndian>(self.address.len() as u64)?;
        writer.write_all(self.address.as_bytes())?;

        writer.write_u64::<LittleEndian>(self.value)?;
        writer.write_all(self.memo.encode().as_array())
    }
}
