// crates
use clap::Parser;

// imports from submodules
use crate::cli::Args;
use crate::convert::convert_dir;

// modules
pub mod biquad;
pub mod cli;
pub mod convert;
pub mod vox;
pub mod wav;

fn main() {
    // batch fn args
    let args = Args::parse();

    // handles all processing
    convert_dir(&args);
}
