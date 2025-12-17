mod cli;

use clap::Parser;
use owo_colors::OwoColorize;
use zecwallet_parser::reader::WalletReader;

use crate::cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    match &cli.command {
        Some(Commands::Summarize) | None => {
            let path = cli.wallet_file;
            let reader = WalletReader::read(path).unwrap();
            println!(
                "Wallet was created for {} {} {}",
                "[".red(),
                reader.chain_name.bright_green(),
                "]\n".red(),
            );
            let key_count =
                reader.keys.okeys.len() + reader.keys.zkeys.len() + reader.keys.tkeys.len();

            if key_count > 0 {
                println!(
                    "{} {:#?} {}\n",
                    "Found".bold(),
                    key_count.bold().red(),
                    "keys:".bold()
                );

                if reader.keys.okeys.len() > 0 {
                    println!(
                        "{} {} {}",
                        "-",
                        "Orchard:".bold().green(),
                        reader.keys.okeys.len().red().bold()
                    );
                }

                if reader.keys.zkeys.len() > 0 {
                    println!(
                        "{} {} {}",
                        "-",
                        "Sapling:".bold().green(),
                        reader.keys.zkeys.len().red().bold()
                    );
                }

                if reader.keys.tkeys.len() > 0 {
                    println!(
                        "{} {} {}",
                        "-",
                        "Transparent:".bold().green(),
                        reader.keys.tkeys.len().red().bold()
                    );
                }
            }
        }
    }
}
