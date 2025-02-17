use std::path::PathBuf;

use clap::Parser;

use crate::image_impls::BorderStyle;

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
    pub output_height: Option<usize>,
    #[arg(long = "width")]
    pub output_width: usize,
    #[arg(long)]
    pub display: bool,
    #[arg(long, default_value_t = 10)]
    pub max_depth: usize,
    #[arg(long, value_enum, default_value_t = BorderStyle::Looping)]
    pub border_style: BorderStyle,
    #[arg(long = "repeat")]
    pub repeating: bool,
}
