use std::env;

use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

pub(crate) fn init_tracing(debug: u8) {
    if debug == 0 && env::var_os("RUST_LOG").is_none() {
        return;
    }

    let level = match debug {
        0 => "info",
        1 => "info",
        _ => "debug",
    };

    let fallback = format!("zecwallet_parser={level},zecwallet_dump={level}");

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(fallback));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::ACTIVE)
        .with_target(false)
        .compact()
        .init();
}
