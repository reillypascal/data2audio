use std::any::TypeId;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use hound::{self, Sample, WavReader, WavSpec, WavWriter};

use crate::cli::Args;

pub fn read_file_as_wav<T>(path: &Path) -> Result<(Vec<T>, WavSpec), hound::Error>
where
    T: Sample,
{
    // ? operator is like match expression for Result
    let mut reader = WavReader::open(path)?;
    let input = reader
        .samples::<T>()
        .collect::<Result<Vec<T>, hound::Error>>()?;
    let spec = reader.spec();
    // return
    Ok((input, spec))
}
// &[i16] instead of &Vec<i16> - https://rust-lang.github.io/rust-clippy/master/index.html#ptr_arg
pub fn write_file_as_wav<T>(data: &[T], path: &PathBuf, args: &Args) -> Result<(), hound::Error>
where
    T: Copy + Sample + 'static, // &TypeId::of<T>() requires 'static
{
    let fmt_num_bits = HashMap::<TypeId, u16>::from([
        (TypeId::of::<i8>(), 8),
        (TypeId::of::<i16>(), 16),
        // (TypeId::of::<i24>(), 24),
        (TypeId::of::<i32>(), 32),
    ]);

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: args.samplerate,
        bits_per_sample: fmt_num_bits[&TypeId::of::<T>()],
        sample_format: hound::SampleFormat::Int,
    };

    //writer
    let mut writer = WavWriter::create(path, spec)?;
    for sample in data {
        writer.write_sample(*sample)?;
    }
    writer.finalize()?;

    Ok(())
}
