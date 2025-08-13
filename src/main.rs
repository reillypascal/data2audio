// use std::path::{self, Path, PathBuf, StripPrefixError};
// use walkdir::{DirEntry, WalkDir};
// std
use std::fs;
use std::path::{self, PathBuf};
// crates
use clap::{Parser, ValueEnum};
use walkdir::WalkDir;

use crate::cli::{Args, SampleFormat};

// modules
pub mod batch;
pub mod biquad;
pub mod cli;
pub mod vox;
pub mod wav;

fn main() {
    // ---- CLI ARGUMENTS ----
    let args = Args::parse();

    // ---- GET & PROCESS FILES ----
    // WalkDir "walks" recursively through a directory and all its subfolders
    WalkDir::new(&args.input)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .for_each(|entry| {
            // extract metadata from Result<T,E> for each entry in dir
            if let Ok(metadata) = entry.metadata() {
                // if it's a file and greater/equal to min size we're looking at
                if metadata.is_file() && metadata.len() >= args.min {
                    // ---- IMPORT FILE ----
                    // import as Vec<u8>
                    let data: Vec<u8> = fs::read(entry.path()).expect("Error reading file");

                    // ---- CONVERT BASED ON SAMPLE FORMAT ----
                    // need to filter as f64 anyway, so best to do in
                    // match arms here for consistency
                    let converted_data: Vec<f64> = match args.format {
                        SampleFormat::Uint8 => {
                            data.iter()
                                .map(|chunk| {
                                    // bit-shift based on using 16-bit wav at output
                                    // need to do as 16-bit to avoid overflow in shift
                                    ((*chunk as u16) << 8) as f64
                                })
                                .collect()
                        }
                        SampleFormat::Int16 => {
                            data.chunks_exact(2)
                                .map(|chunks| {
                                    // from_le_bytes() takes array of bytes and converts to a single little-endian integer
                                    i16::from_le_bytes(
                                        chunks.try_into().expect("Could not import as 16-bit"),
                                    ) as f64
                                })
                                .collect()
                        }
                        SampleFormat::Int24 => {
                            data.chunks_exact(3)
                                .map(|chunks| {
                                    // get values from chunks_exact(3), put in array
                                    let low_part: [u8; 3] =
                                        chunks.try_into().expect("Could not import as 24-bit");
                                    // no i24, so we add this 0x00 to fill out hi byte in i32
                                    let high_part: [u8; 1] = [0x00];
                                    // copy to "joined" from low/hi parts as slices
                                    let mut joined: [u8; 4] = [0; 4];
                                    joined[3..].copy_from_slice(&high_part);
                                    joined[..3].copy_from_slice(&low_part);

                                    (i32::from_le_bytes(joined) >> 8) as f64
                                })
                                .collect()
                        }
                        SampleFormat::Int32 => {
                            data.chunks_exact(4)
                                .map(|chunks| {
                                    // bit-shift based on using 16-bit wav at output
                                    (i32::from_le_bytes(
                                        chunks.try_into().expect("Could not import as 32-bit"),
                                    ) >> 16) as f64
                                })
                                .collect()
                        }
                        SampleFormat::Vox => {
                            let mut output: Vec<f64> = Vec::new();
                            let mut vox_state = vox::VoxState::new();
                            data.iter()
                                // using for_each and...
                                .for_each(|chunk| {
                                    // start with highest 4 bits (by right-shifting); & 0b1111 selects lowest 4
                                    for nibble in [chunk >> 4, chunk & 0b1111].iter() {
                                        output.push(vox_state.vox_decode(nibble) as f64);
                                    }
                                });
                            // ...returning outside of pipeline since we need to handle *two* nibbles per element in iter()
                            output
                        }
                    };

                    // ---- FILTERING ----
                    // make filter
                    let mut filter = biquad::AudioFilter::new();
                    filter.calculate_filter_coeffs();
                    // vec in which to process sound
                    let mut filtered_vec = Vec::<i16>::new();
                    // let mut unfiltered_vec = Vec::<i16>::new();
                    // filter audio
                    for sample in &converted_data {
                        let filtered_samp = filter.process_sample(*sample * 0.4);
                        filtered_vec.push(filtered_samp as i16);
                        // unfiltered_vec.push(*sample as i16);
                    }

                    // ---- OUTPUT FILE ----
                    // write all files into output directory
                    let mut write_path = PathBuf::from(&args.output);
                    // create output dir if doesn't exist - create_dir returns Result<T,E>, so match it and print if err
                    let out_dir = create_dir(&args.output);
                    match out_dir {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("{e}")
                        }
                    };
                    // entry.path().file_name() returns an Option, so if let Some() handles/extracts value
                    if let Some(file_name) = entry.path().file_name() {
                        write_path.push(file_name);
                        write_path.set_extension("wav");
                        write_file_as_wav(filtered_vec, write_path);
                        // write_file_as_wav(unfiltered_vec, write_path);
                    }
                }
            }
        });
}
/*
// ---- CLI PARSER ----
#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 'i', long, default_value_t = String::from("input"))]
    input: String,

    #[arg(short = 'o', long, default_value_t = String::from("output"))]
    output: String,

    #[arg(short = 'm', long, default_value_t = 0)]
    min: u64,

    #[clap(short = 'f', long, value_enum, default_value_t=SampleFormat::Int16)]
    format: SampleFormat,

    #[arg(short = 'F', long, default_value_t = true)]
    filter: bool,
}

#[derive(ValueEnum, Clone, Debug)]
enum SampleFormat {
    Uint8,
    Int16,
    Int24,
    Int32,
    Vox,
}
*/
// ---- WRITING WAVs ----
fn create_dir(dir: &str) -> std::io::Result<()> {
    // create_dir_all - like multiple mkdir calls
    fs::create_dir_all(dir)?;
    Ok(())
}

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
    for sample in data {
        writer.write_sample(sample).expect("Could not write sample");
    }
    writer.finalize().expect("Could not finalize WAV file");
}
