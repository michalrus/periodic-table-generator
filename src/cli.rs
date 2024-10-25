use clap::Parser;

/// Periodic table generator that can be used to generate SVGs for spaced-repetition study in Anki.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Hide element symbols
    #[arg(long)]
    pub no_symbols: bool,

    /// Draw it wide instead of lanthanoids and actinoids separately
    #[arg(long)]
    pub wide: bool,

    /// Draw helium in group 2 instead of 18 (for electron configurations)
    #[arg(long)]
    pub helium_in_2: bool,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }
}
