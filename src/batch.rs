use std::fs;
use std::path::PathBuf;

use rayon::prelude::*;
use walkdir::WalkDir;

use crate::biquad::{AudioFilter, AudioFilterParameters, FilterAlgorithm};
use crate::cli::{Args, SampleFormat};
use crate::vox;
use crate::wav::write_file_as_wav;

pub fn process_batch(args: &Args) {
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
                let mut data: Vec<u8> = vec![];
                match fs::read(entry.path()) {
                    Ok(file) => {
                        data = file;
                    }
                    Err(e) => {
                        eprintln!("Error reading {:?} as .WAV file: {}", entry.path(), e);
                    }
                };
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
                // only filter if filter arg is set true
                let output_vec: Vec<i16> = match args.filter {
                    true => {
                        // make filter
                        let filter_params =
                            AudioFilterParameters::new(FilterAlgorithm::Hpf2, 20.0, 0.707, 0.0);
                        let mut filter = AudioFilter::new(&filter_params, args.samplerate);
                        filter.calculate_filter_coeffs();
                        let gain_lin = f64::powf(10.0, args.gain / 20.0);
                        // filter audio
                        converted_data
                            .iter()
                            .map(|sample| filter.process_sample(*sample * gain_lin) as i16)
                            .collect()
                    }
                    false => converted_data.iter().map(|sample| *sample as i16).collect(),
                };

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

                    let spec = hound::WavSpec {
                        channels: 1,
                        sample_rate: args.samplerate,
                        bits_per_sample: 16,
                        sample_format: hound::SampleFormat::Int,
                    };

                    match write_file_as_wav(&output_vec, &write_path, &spec) {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("{e}")
                        }
                    };
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
