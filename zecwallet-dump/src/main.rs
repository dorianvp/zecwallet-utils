use std::path::PathBuf;

use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;
use parser::reader::WalletReader;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The ZecWallet Lite wallet file.
    wallet_file: PathBuf,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Summarizes the contents of the specified ZecWallet Lite wallet file.
    Summarize,
}

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
