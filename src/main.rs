use std::fs;
use hound;

fn main() {
    let data: Vec<u8> = fs::read("input/OFXLoader").expect("Error");
    
    let mut converted_data: Vec<i16> = Vec::new();
    
    let iter = data.chunks_exact(2);
    
    for item in iter {
        converted_data.push(i16::from_le_bytes(item.try_into().unwrap()));
    }
    
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = hound::WavWriter::create("output/OFXLoader.wav", spec).expect("Could not create writer");
    
    for t in 0..converted_data.len() {
        writer.write_sample(converted_data[t]).expect("Could not write sample");
    }
    
    writer.finalize().expect("Could not finalize WAV file");
    
    // let s = format!("{:?}", &data);
    // println!("{}", s);
}
