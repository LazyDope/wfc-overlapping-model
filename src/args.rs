use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
pub(crate) struct Args {
    #[arg(short, long)]
    pub(crate) input: PathBuf,
    #[arg(short, long, default_value_t = 3)]
    pub(crate) tile_size: u32,
    #[arg(long = "height")]
    pub(crate) output_height: usize,
    #[arg(long = "width")]
    pub(crate) output_width: usize,
}
