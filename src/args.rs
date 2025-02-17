use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    #[arg(short, long)]
    pub input: PathBuf,
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    #[arg(short, long, default_value_t = 3)]
    pub tile_size: u32,
    #[arg(long = "height")]
    pub output_height: usize,
    #[arg(long = "width")]
    pub output_width: usize,
    #[arg(long)]
    pub display: bool,
    #[arg(long, default_value_t = 10)]
    pub max_depth: usize,
}
