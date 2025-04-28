// use std::{env, fs};
use std::fs;
// use std::path::{self, Path, PathBuf, StripPrefixError}; 
use std::path::{self, PathBuf};
use clap::{Parser, Subcommand};
use hound;
// use walkdir::{DirEntry, WalkDir};
use walkdir::WalkDir;

pub mod biquad;

fn main() {
    // let current_dir = env::current_dir().expect("Error getting directory");
    
    // ---- CLI ARGUMENTS ----
    let args = Args::parse();
    // if/else is expression, can be assigned
    let in_path = if let Some(path) = args.in_path {
        path
    } else {
        String::from("input")
    };
    
    // ---- GET & PROCESS FILES ----
    // read dir
    WalkDir::new(in_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .for_each(|entry| {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    // ---- IMPORT FILE ----
                    // import as Vec<u8>
                    let data: Vec<u8> = fs::read(entry.path()).expect("Error reading file");
                    // convert to Vec<i16>
                    let mut converted_data: Vec<i16> = Vec::new();
                    
                    let iter = data.chunks_exact(2);
                    for item in iter {
                        converted_data.push(i16::from_le_bytes(item.try_into().expect("Could not convert to 16-bit")));
                    }
                    
                    // ---- FILTERING ----
                    // make filter
                    let mut filter = biquad::AudioFilter::new();
                    filter.calculate_filter_coeffs();
                    // vec in which to process sound
                    let mut filtered_vec = Vec::<i16>::new();
                    // filter audio
                    for sample in &converted_data {
                        let float_samp = *sample as f64;
                        let filtered_samp = filter.process_sample(float_samp * 0.4);
                        filtered_vec.push(filtered_samp as i16);
                    }
                    
                    // ---- OUTPUT FILE ----
                    // write all files into output directory
                    let mut write_path = PathBuf::from("output/");
                    // entry.path().file_name() returns an Option
                    if let Some(file_name) = entry.path().file_name() {
                        write_path.push(file_name);
                        write_path.set_extension("wav");
                        write_file_as_wav(filtered_vec, write_path);
                    }
                }
            }
        });
}

// fn replace_prefix(p: impl AsRef<Path>, from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<PathBuf, StripPrefixError> {
//   p.as_ref().strip_prefix(from).map(|p| to.as_ref().join(p))
// }

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    in_path: Option<String>,
    out_path: Option<String>,
    // #[command(subcommand)]
    // cmd: Commands
}

// #[derive(Subcommand, Debug, Clone)]
// enum Commands {
//     Min(String),
// }

fn write_file_as_wav(data: Vec<i16>, name: path::PathBuf) {
    // write WAV file
    // spec
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    // writer
    let mut writer = hound::WavWriter::create(name, spec).expect("Could not create writer");
    for t in 0..data.len() {
        writer.write_sample(data[t]).expect("Could not write sample");
    }
    writer.finalize().expect("Could not finalize WAV file");
}
