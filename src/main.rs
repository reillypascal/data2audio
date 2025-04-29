// use std::{env, fs};
use std::fs;
// use std::path::{self, Path, PathBuf, StripPrefixError}; 
use std::path::{self, PathBuf};
// use clap::{Parser, ArgEnum};
use clap::{Parser, ValueEnum};
use hound;
// use walkdir::{DirEntry, WalkDir};
use walkdir::WalkDir;

pub mod biquad;

fn main() {
    // let current_dir = env::current_dir().expect("Error getting directory");
    
    // ---- CLI ARGUMENTS ----
    let args = Args::parse();
    // if/else is expression, can be assigned
    
    // ---- GET & PROCESS FILES ----
    // read dir - input as ref so don't move
    WalkDir::new(&args.input)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .for_each(|entry| {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    // ---- IMPORT FILE ----
                    // import as Vec<u8>
                    let data: Vec<u8> = fs::read(entry.path()).expect("Error reading file");
                    
                    // // convert to Vec<i16>
                    // let mut converted_data: Vec<i16> = Vec::new();
                    
                    // let iter = data.chunks_exact(2);
                    // for item in iter {
                    //     converted_data.push(i16::from_le_bytes(item.try_into().expect("Could not convert to 16-bit")));
                    // }
                    
                    // ---- CONVERT BASED ON SAMPLE FORMAT ----
                    // need to filter as f64 anyway, so best to do in match arms here for consistency
                    let converted_data: Vec<f64> = match args.format {
                        SampleFormat::Uint8 => {
                            data
                                .iter()
                                .map(|chunk| {
                                    ((*chunk as u16) << 8) as f64
                                }).collect()
                        }
                        SampleFormat::Int16 => {
                            data
                                .chunks_exact(2)
                                .map(|chunks| {
                                    i16::from_le_bytes(chunks.try_into().expect("Could not import as 16-bit")) as f64
                                }).collect()
                        }
                        SampleFormat::Int24 => {
                            data
                                .chunks_exact(3)
                                .map(|chunks| {
                                    let low_part: [u8; 3] = chunks.try_into().expect("Could not import as 24-bit");
                                    let high_part: [u8; 1] = [0x00];
                                    let mut joined: [u8; 4] = [0; 4];
                                    
                                    joined[3..].copy_from_slice(&high_part);
                                    joined[..3].copy_from_slice(&low_part);
                                    
                                    (i32::from_le_bytes(joined) >> 8) as f64
                                }).collect()
                        }
                        SampleFormat::Int32 => {
                            data
                                .chunks_exact(4)
                                .map(|chunks| {
                                    (i32::from_le_bytes(chunks.try_into().expect("Could not import as 32-bit")) >> 16) as f64
                                }).collect()
                        }
                    };
                    
                    // ---- FILTERING ----
                    // make filter
                    let mut filter = biquad::AudioFilter::new();
                    filter.calculate_filter_coeffs();
                    // vec in which to process sound
                    let mut filtered_vec = Vec::<i16>::new();
                    // filter audio
                    for sample in &converted_data {
                        let filtered_samp = filter.process_sample(*sample * 0.4);
                        filtered_vec.push(filtered_samp as i16);
                    }
                    
                    // ---- OUTPUT FILE ----
                    // write all files into output directory
                    // args.output as ref so don't move
                    let mut write_path = PathBuf::from(&args.output);
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

// ---- CLI PARSER ----
#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 'i', long, default_value_t = String::from("input"))]
    input: String,
    
    #[arg(short = 'o', long, default_value_t = String::from("output"))]
    output: String,
    
    #[arg(short = 'm', long, default_value_t = 0)]
    min: usize,
    
    #[clap(short = 'f', long, value_enum, default_value_t=SampleFormat::Int16)]
    format: SampleFormat,
}

#[derive(ValueEnum, Clone, Debug)]
enum SampleFormat {
    Uint8,
    Int16,
    Int24,
    Int32,
    // Float32,
    // Float64,
}

// ---- WAV WRITER ----
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
