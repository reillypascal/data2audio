use clap::{Parser, ValueEnum};
//use hound::SampleFormat;

// ---- CLI PARSER ----
#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short = 'i', long, default_value_t = String::from("input"))]
    pub input: String,

    #[arg(short = 'o', long, default_value_t = String::from("output"))]
    pub output: String,

    #[arg(short = 'm', long, default_value_t = 0)]
    pub min: u64,

    #[clap(short = 'f', long, value_enum, default_value_t=SampleFormat::Int16)]
    pub format: SampleFormat,

    #[arg(short = 'F', long, default_value_t = true)]
    pub filter: bool,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum SampleFormat {
    Uint8,
    Int16,
    Int24,
    Int32,
    Vox,
}
