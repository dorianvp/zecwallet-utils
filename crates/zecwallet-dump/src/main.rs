mod cli;
mod summary;

use std::process;

use clap::Parser;
use zecwallet_parser::reader::WalletReader;

use crate::cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    let wallet = match WalletReader::read(&cli.wallet_file) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Error reading wallet: {e:?}");
            process::exit(1);
        }
    };

    match &cli.command {
        Some(Commands::Summarize) | None => {
            summary::print_summary(&wallet, cli.debug);
        }
    }
}
