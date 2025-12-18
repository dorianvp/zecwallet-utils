#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verbosity {
    Basic,
    Verbose,
    Debug,
}

#[derive(Copy, Clone, Debug)]
pub struct OutputOptions {
    pub verbosity: Verbosity,
    pub show_secrets: bool,
}
