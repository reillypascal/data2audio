use std::ffi::OsStr;
use std::path::PathBuf;

use hound::WavSpec;
use walkdir::{DirEntry, WalkDir};

use crate::cli::Args;
use crate::wav::{read_file_as_wav, write_file_as_wav};

pub fn process_batch(args: &Args, wav_spec: &mut WavSpec) {
    WalkDir::new(&args.input)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .for_each(|entry| {});
}
