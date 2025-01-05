use clap::Parser;

#[derive(Parser)]
#[command(name = "Transaction Decoder")]
#[command(version = "1.0")]
#[command(about = "Bitcoin Transaction Decoder", long_about = None)]
struct Cli {
    transaction_hex: String,
}

fn main() {
    let cli = Cli::parse();

    match rust_transaction_decoder::decode(cli.transaction_hex) {
        Ok(json) => println!("{}", json),
        Err(e) => println!("{}", e),
    }
}
