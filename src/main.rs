use std::fs;
use hound;

fn main() {
    let data: Vec<u8> = fs::read("input/OFXLoader").expect("Error");
    
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = hound::WavWriter::create("output/OFXLoader.wav", spec).expect("Could not create writer");
    
    for t in 0..data.len() {
        writer.write_sample(data[t] as i16).expect("Could not write sample");
    }
    
    writer.finalize().expect("Could not finalize WAV file");
    
    // let s = format!("{:?}", &data);
    // println!("{}", s);
}
