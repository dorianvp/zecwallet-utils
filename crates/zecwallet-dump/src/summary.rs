use owo_colors::OwoColorize;
use zecwallet_parser::zwl::ZwlWallet;

/// Prints the summary of the given wallet.
/// Output varies whether in `debug` mode or `standard` mode.
///
/// # Standard Mode
///
/// The summary consists of the following data:
///
/// - File version
/// - Network
/// - Mnemonic
/// - Birthday
/// - Latest sync height (verified tree)
/// - Number of keys, specified per pool type
/// - Transactions:
///   - Total count
///   - Range of block heights
///   - Estimated balance
///
/// # Debug Mode
///
/// TODO
///
pub fn print_summary(wallet: &ZwlWallet, _debug: u8) {
    print_header(wallet);
    print_key_summary(wallet);
}

fn print_header(wallet: &ZwlWallet) {
    println!(
        "Wallet was created for {} {} {}",
        "[".red(),
        wallet.chain_name.bright_green(),
        "]\n".red(),
    );
}

fn print_key_summary(wallet: &ZwlWallet) {
    let key_count = wallet.keys.okeys.len() + wallet.keys.zkeys.len() + wallet.keys.tkeys.len();

    if key_count == 0 {
        println!("No keys found in wallet.");
        return;
    }

    println!(
        "{} {:#?} {}\n",
        "Found".bold(),
        key_count.bold().red(),
        "keys:".bold()
    );

    if !wallet.keys.okeys.is_empty() {
        println!(
            "{} {} {}",
            "-",
            "Orchard:".bold().green(),
            wallet.keys.okeys.len().red().bold()
        );
    }

    if !wallet.keys.zkeys.is_empty() {
        println!(
            "{} {} {}",
            "-",
            "Sapling:".bold().green(),
            wallet.keys.zkeys.len().red().bold()
        );
    }

    if !wallet.keys.tkeys.is_empty() {
        println!(
            "{} {} {}",
            "-",
            "Transparent:".bold().green(),
            wallet.keys.tkeys.len().red().bold()
        );
    }
}
