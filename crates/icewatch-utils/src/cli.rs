use clap::Parser;
use tracing::level_filters::LevelFilter;

#[derive(Parser, Debug)]
pub struct CmdArgs {
    /// Logger verbosity
    #[clap(short, long)]
    pub verbosity: Option<LevelFilter>,
    #[clap(long)]
    pub log_to_file: bool,
}

pub fn parse() -> CmdArgs {
    CmdArgs::parse()
}
