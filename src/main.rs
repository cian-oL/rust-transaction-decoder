mod transaction;

use clap::Parser;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::io::{Error as IoError, Read};

use transaction::*;

#[derive(Parser)]
#[command(name = "Transaction Decoder")]
#[command(version = "1.0")]
#[command(about = "Bitcoin Transaction Decoder", long_about = None)]
struct Cli {
    transaction_hex: String,
}

fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, IoError> {
    let mut compact_size = [0_u8; 1];
    transaction_bytes.read(&mut compact_size)?;

    match compact_size[0] {
        0..=252 => Ok(compact_size[0] as u64),
        253 => {
            let mut buffer = [0; 2];
            transaction_bytes.read(&mut buffer)?;
            Ok(u16::from_le_bytes(buffer) as u64)
        }
        254 => {
            let mut buffer = [0; 4];
            transaction_bytes.read(&mut buffer)?;
            Ok(u32::from_le_bytes(buffer) as u64)
        }
        255 => {
            let mut buffer = [0; 8];
            transaction_bytes.read(&mut buffer)?;
            Ok(u64::from_le_bytes(buffer))
        }
    }
}

#[allow(unused_variables)]
fn read_u32(transaction_bytes: &mut &[u8]) -> Result<u32, IoError> {
    let mut buffer = [0; 4];
    transaction_bytes.read(&mut buffer)?;

    Ok(u32::from_le_bytes(buffer))
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, IoError> {
    let mut buffer = [0; 8];
    transaction_bytes.read(&mut buffer)?;

    Ok(Amount::from_sat(u64::from_le_bytes(buffer)))
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, IoError> {
    let mut buffer = [0; 32];
    transaction_bytes.read(&mut buffer)?;

    Ok(Txid::from_bytes(buffer))
}

fn read_script(transaction_bytes: &mut &[u8]) -> Result<String, IoError> {
    let script_size = read_compact_size(transaction_bytes)? as usize;
    let mut buffer = vec![0_u8; script_size];
    transaction_bytes.read(&mut buffer)?;

    Ok(hex::encode(buffer))
}

fn hash_raw_transaction(raw_transaction: &[u8]) -> Txid {
    let mut hasher = Sha256::new();
    hasher.update(&raw_transaction);
    let hash1 = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(hash1);
    let hash2 = hasher.finalize();

    Txid::from_bytes(hash2.into())
}

fn decode(transaction_hex: String) -> Result<String, Box<dyn Error>> {
    let transaction_bytes =
        hex::decode(transaction_hex).map_err(|e| format!("Hex decode error: {}", e))?;
    let mut bytes_slice = transaction_bytes.as_slice();

    // decode version
    let version = read_u32(&mut bytes_slice)?;

    // decode inputs
    let input_count = read_compact_size(&mut bytes_slice)?;
    let mut inputs = vec![];

    for _ in 0..input_count {
        let txid = read_txid(&mut bytes_slice)?;
        let output_index = read_u32(&mut bytes_slice)?;
        let script_sig = read_script(&mut bytes_slice)?;
        let sequence = read_u32(&mut bytes_slice)?;

        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
        });
    }

    // decode outputs
    let output_count = read_compact_size(&mut bytes_slice)?;
    let mut outputs = vec![];

    for _ in 0..output_count {
        let amount = read_amount(&mut bytes_slice)?;
        let script_pubkey = read_script(&mut bytes_slice)?;

        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }

    // decode locktime
    let lock_time = read_u32(&mut bytes_slice)?;
    let transaction_id = hash_raw_transaction(&transaction_bytes);

    // initialise decoded transaction
    let transaction = Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time,
    };

    Ok(serde_json::to_string_pretty(&transaction)?)
}

fn main() {
    let cli = Cli::parse();

    match decode(cli.transaction_hex) {
        Ok(json) => println!("{}", json),
        Err(e) => println!("{}", e),
    }
}

#[cfg(test)]
mod test {
    use super::read_compact_size;
    use super::Error;

    #[test]
    fn test_read_compact_size() -> Result<(), Box<dyn Error>> {
        let mut bytes = [1_u8].as_slice();
        let count = read_compact_size(&mut bytes)?;
        assert_eq!(count, 1_u64);

        let mut bytes = [253_u8, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes)?;
        assert_eq!(count, 256_u64);

        let mut bytes = [254_u8, 0, 0, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes)?;
        assert_eq!(count, 256_u64.pow(3));

        let mut bytes = [255_u8, 0, 0, 0, 0, 0, 0, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes)?;
        assert_eq!(count, 256_u64.pow(7));

        let hex = "fd204e";
        let decoded = hex::decode(hex)?;
        let mut bytes = decoded.as_slice();
        let count = read_compact_size(&mut bytes)?;
        let expected_count = 20_000_u64;
        assert_eq!(count, expected_count);

        Ok(())
    }
}
