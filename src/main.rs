// use std::{env, fs};
use std::fs;
use std::path::{self, Path, PathBuf, StripPrefixError}; 
use hound;
use walkdir::{DirEntry, WalkDir};

fn main() {
    // let current_dir = env::current_dir().expect("Error getting directory");
    // read dir
    WalkDir::new("input/")
        .into_iter()
        .filter_map(|entry| entry.ok())
        .for_each(|entry| {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    // import file as Vec<u8>
                    let data: Vec<u8> = fs::read(entry.path()).expect("Error reading file");
                    
                    // convert to Vec<i16>
                    let mut converted_data: Vec<i16> = Vec::new();
                    
                    let iter = data.chunks_exact(2);
                    for item in iter {
                        converted_data.push(i16::from_le_bytes(item.try_into().unwrap()));
                    }
                    
                    // write all files into output directory
                    let mut write_path = PathBuf::from("output/");
                    
                    // entry.path().file_name() returns an Option
                    if let Some(file_name) = entry.path().file_name() {
                        write_path.push(file_name);
                        write_path.set_extension("wav");
                        write_file_as_wav(converted_data, write_path);
                    }
                }
            }
        });
}

// fn replace_prefix(p: impl AsRef<Path>, from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<PathBuf, StripPrefixError> {
//   p.as_ref().strip_prefix(from).map(|p| to.as_ref().join(p))
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
