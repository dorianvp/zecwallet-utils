use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

pub(crate) fn init_tracing(debug: u8) {
    let level = match debug {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    let fallback = format!("zecwallet_parser={level},zecwallet_dump={level}");

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(fallback));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::FULL)
        .with_target(false)
        .compact()
        .init();
}
