use clap::{Parser, ValueEnum};
// use std::collections::HashMap;
// use std::sync::LazyLock;

// ---- CLI PARSER ----
#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short = 'i', long, default_value_t = String::from("input"))]
    pub input: String,

    #[arg(short = 'o', long, default_value_t = String::from("output"))]
    pub output: String,

    #[arg(short = 'm', long, default_value_t = 0)]
    pub min: u64,

    #[arg(short = 's', long, default_value_t = 44100)]
    pub samplerate: u32,

    #[clap(short = 'f', long, value_enum, default_value_t=SampleFormat::Int16)]
    pub format: SampleFormat,

    #[clap(short = 'e', long, value_enum, default_value_t=Endianness::Little)]
    pub endian: Endianness,
    // #[clap(short = 'F', long, value_enum)]
    // pub out_format: Option<SampleFormat>,
    #[arg(short = 'r', long, default_value_t = false)]
    pub raw: bool,

    #[arg(short = 'g', long, default_value_t = -8.0)]
    pub gain: f64,
}

#[derive(ValueEnum, Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub enum SampleFormat {
    Int8,
    Int16,
    Int24,
    Int32,
    Vox,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Endianness {
    Little,
    Big,
}
// pub const FMT_NUM_BITS: LazyLock<HashMap<SampleFormat, u16>> = LazyLock::new(|| {
//     HashMap::from([
//         (SampleFormat::Uint8, 8),
//         (SampleFormat::Int16, 16),
//         (SampleFormat::Int24, 24),
//         (SampleFormat::Int32, 32),
//         (SampleFormat::Vox, 8),
//     ])
// });
