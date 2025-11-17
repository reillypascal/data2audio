use std::fs;
use std::path::PathBuf;

use rayon::prelude::*;
use walkdir::WalkDir;

use crate::biquad::{AudioFilter, AudioFilterParameters, FilterAlgorithm};
use crate::cli::{Args, Endianness, SampleFormat};
use crate::vox;
use crate::wav::write_file_as_wav;

pub fn convert_dir(args: &Args) {
    WalkDir::new(&args.input)
        .into_iter()
        .par_bridge() // .par_bridge() is less effective than .into_par_iter(),
        // but hard to parallelize file I/O with regular par iter
        .filter_map(|entry| entry.ok())
        .for_each(|entry| {
            if let Ok(metadata) = entry.metadata()
                && metadata.is_file()
                && metadata.len() >= args.min
            {
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

                    write_path.set_extension("");

                    if !(&args.append.is_empty()) {
                        write_path = append_to_path(write_path, &args.append);
                    }

                    // write_path.set_extension("wav");
                    // using append prevents removing non-extension dot-separated
                    // parts (which would also remove appended, if it exists)
                    write_path = append_to_path(write_path, ".wav");

                    let mut data: Vec<u8> = vec![];
                    match fs::read(entry.path()) {
                        Ok(file) => {
                            data = file;
                        }
                        Err(e) => {
                            eprintln!("Error reading {:?} as .WAV file: {}", entry.path(), e);
                        }
                    };

                    // make filter
                    let filter_params =
                        AudioFilterParameters::new(FilterAlgorithm::Hpf2, 20.0, 0.707, 0.0);
                    let mut filter = AudioFilter::new(&filter_params, args.samplerate);
                    filter.calculate_filter_coeffs();
                    let gain_lin = f64::powf(10.0, args.gain / 20.0);

                    // ---- CONVERT BASED ON SAMPLE FORMAT ----
                    match args.format {
                        SampleFormat::Int8 => {
                            let mut formatted_data: Vec<i8> = data
                                .iter()
                                // needs to be i8 to satisfy Sample trait bound
                                .map(|chunk| ((*chunk as i16) - 128) as i8)
                                .collect();

                            if !args.raw {
                                for sample in &mut formatted_data {
                                    *sample =
                                        (filter.process_sample((*sample as f64) * gain_lin)) as i8;
                                }
                            }

                            match write_file_as_wav(&formatted_data, &write_path, args) {
                                Ok(()) => {}
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            };
                        }
                        SampleFormat::Int16 => {
                            let mut formatted_data: Vec<i16> = data
                                .chunks_exact(2)
                                .map(|chunk| {
                                    match &args.endian {
                                        Endianness::Big => {
                                            // from_le_bytes() takes array of bytes and converts to a single little-endian integer
                                            i16::from_be_bytes(
                                                chunk
                                                    .try_into()
                                                    .expect("Could not import as 16-bit"),
                                            )
                                        }
                                        Endianness::Little => {
                                            // from_le_bytes() takes array of bytes and converts to a single little-endian integer
                                            i16::from_le_bytes(
                                                chunk
                                                    .try_into()
                                                    .expect("Could not import as 16-bit"),
                                            )
                                        }
                                    }
                                })
                                .collect();

                            if !args.raw {
                                for sample in &mut formatted_data {
                                    *sample =
                                        (filter.process_sample((*sample as f64) * gain_lin)) as i16;
                                }
                            }

                            match write_file_as_wav(&formatted_data, &write_path, args) {
                                Ok(()) => {}
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            };
                        }
                        SampleFormat::Int24 => {
                            let mut formatted_data: Vec<i32> = data
                                .chunks_exact(3)
                                .map(|chunk| {
                                    // get values from chunks_exact(3), put in array
                                    let data_bytes: [u8; 3] =
                                        chunk.try_into().expect("Could not import as 24-bit");
                                    // no i24, so we add this 0x00 to fill out hi byte in i32
                                    let padding_byte: [u8; 1] = [0x00];
                                    // copy to "joined" from low/hi parts as slices
                                    let mut joined: [u8; 4] = [0; 4];

                                    match &args.endian {
                                        Endianness::Big => {
                                            joined[..1].copy_from_slice(&padding_byte);
                                            joined[1..].copy_from_slice(&data_bytes);

                                            i32::from_be_bytes(joined)
                                        }
                                        Endianness::Little => {
                                            joined[3..].copy_from_slice(&padding_byte);
                                            joined[..3].copy_from_slice(&data_bytes);

                                            i32::from_le_bytes(joined)
                                        }
                                    }
                                })
                                .collect();

                            if !args.raw {
                                for sample in &mut formatted_data {
                                    *sample =
                                        (filter.process_sample((*sample as f64) * gain_lin)) as i32;
                                }
                            }

                            match write_file_as_wav(&formatted_data, &write_path, args) {
                                Ok(()) => {}
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            };
                        }
                        SampleFormat::Int32 => {
                            let mut formatted_data: Vec<i32> = data
                                .chunks_exact(4)
                                .map(|chunk| {
                                    match &args.endian {
                                        Endianness::Big => {
                                            // bit-shift based on using 16-bit wav at output
                                            i32::from_le_bytes(
                                                chunk
                                                    .try_into()
                                                    .expect("Could not import as 32-bit"),
                                            )
                                        }
                                        Endianness::Little => {
                                            // bit-shift based on using 16-bit wav at output
                                            i32::from_le_bytes(
                                                chunk
                                                    .try_into()
                                                    .expect("Could not import as 32-bit"),
                                            )
                                        }
                                    }
                                })
                                .collect();

                            if !args.raw {
                                for sample in &mut formatted_data {
                                    *sample =
                                        (filter.process_sample((*sample as f64) * gain_lin)) as i32;
                                }
                            }

                            match write_file_as_wav(&formatted_data, &write_path, args) {
                                Ok(()) => {}
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            };
                        }
                        SampleFormat::Vox => {
                            let mut formatted_data: Vec<i16> = Vec::new();
                            let mut vox_state = vox::VoxState::new();
                            data.iter()
                                // using for_each and...
                                .for_each(|chunk| {
                                    // start with highest 4 bits by right-shifting
                                    // & 0b1111 selects lowest 4
                                    for nibble in [chunk >> 4, chunk & 0b1111].iter() {
                                        formatted_data.push(vox_state.vox_decode(nibble));
                                    }
                                });

                            if !args.raw {
                                for sample in &mut formatted_data {
                                    *sample =
                                        (filter.process_sample((*sample as f64) * gain_lin)) as i16;
                                }
                            }

                            match write_file_as_wav(&formatted_data, &write_path, args) {
                                Ok(()) => {}
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            };
                        }
                    };

                    // // ---- FILTERING ----
                    // // only filter if filter arg is set true
                    // let output_vec: Vec<i16> = match args.filter {
                    //     true => {
                    //         // make filter
                    //         let filter_params =
                    //             AudioFilterParameters::new(FilterAlgorithm::Hpf2, 20.0, 0.707, 0.0);
                    //         let mut filter = AudioFilter::new(&filter_params, args.samplerate);
                    //         filter.calculate_filter_coeffs();
                    //         let gain_lin = f64::powf(10.0, args.gain / 20.0);
                    //         // filter audio
                    //         converted_data
                    //             .iter()
                    //             .map(|sample| filter.process_sample(*sample * gain_lin) as i16)
                    //             .collect()
                    //     }
                    //     false => converted_data.iter().map(|sample| *sample as i16).collect(),
                    // };
                }
            }
        });
}

// ---- WRITING WAVs ----
fn create_dir(dir: &str) -> std::io::Result<()> {
    // create_dir_all - like multiple mkdir calls
    fs::create_dir_all(dir)?;
    Ok(())
}

// ---- APPEND TO WRITE PATH ----
// https://stackoverflow.com/a/76378247
fn append_to_path(p: PathBuf, s: &str) -> PathBuf {
    let mut p = p.into_os_string();
    p.push(s);
    p.into()
}
