use serde::{ser::SerializeStruct, Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::{fmt, io::BufRead};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) => write!(f, "IO Error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub struct Transaction {
    pub version: u32,
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<TxOut>,
    pub lock_time: u32,
}

impl Transaction {
    pub fn txid(&self) -> Txid {
        // TODO: proper implementation
        let txid_data = vec![0; 32];
        Txid::new(txid_data)
    }
}

impl Serialize for Transaction {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut tx = serializer.serialize_struct("Transaction", 5)?;
        tx.serialize_field("transaction id", &self.txid())?;
        tx.serialize_field("version", &self.version)?;
        tx.serialize_field("inputs", &self.inputs)?;
        tx.serialize_field("outputs", &self.outputs)?;
        tx.serialize_field("lock_time", &self.lock_time)?;
        tx.end()
    }
}

#[derive(Debug, Serialize)]
pub struct TxIn {
    pub txid: Txid,
    pub output_index: u32,
    pub script_sig: String, // TODO: remove explicit string typing
    pub sequence: u32,
}

#[derive(Debug, Serialize)]
pub struct TxOut {
    #[serde(serialize_with = "as_btc")]
    pub amount: Amount,
    pub script_pubkey: String,
}

#[derive(Debug)]
pub struct Amount(u64);

impl Amount {
    pub fn from_sat(satoshi: u64) -> Amount {
        Amount(satoshi)
    }
}

trait BitcoinValue {
    fn to_btc(&self) -> f64;
}

impl BitcoinValue for Amount {
    fn to_btc(&self) -> f64 {
        self.0 as f64 / 100_000_000.0
    }
}

#[derive(Debug)]
pub struct Txid([u8; 32]);

impl Txid {
    pub fn from_bytes(bytes: [u8; 32]) -> Txid {
        Txid(bytes)
    }

    fn from_raw_transaction(tx: Vec<u8>) -> Txid {
        let mut hasher = Sha256::new();
        hasher.update(&tx);
        let hash1 = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(hash1);
        let hash2 = hasher.finalize();

        Txid::from_bytes(hash2.into())
    }
}

impl Serialize for Txid {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut bytes = self.0.clone();
        bytes.reverse();

        s.serialize_str(&hex::encode(bytes))
    }
}

pub trait Decodable: Sized {
    fn consensus_decode<R: BufRead + ?Sized>(r: &mut R) -> Result<Self, Error>;
}

impl Decodable for u32 {
    fn consensus_decode<R: BufRead + ?Sized>(r: &mut R) -> Result<Self, Error> {
        let mut buffer = [0; 4];
        r.read_exact(&mut buffer).map_err(Error::Io)?;

        Ok(u32::from_le_bytes(buffer))
    }
}

// start at 11:40 impl decodeable for other types

#[derive(Debug, Serialize)]
pub struct CompactSize(pub u64);

fn as_btc<S: Serializer, T: BitcoinValue>(t: &T, s: S) -> Result<S::Ok, S::Error> {
    let btc = t.to_btc();
    s.serialize_f64(btc)
}
