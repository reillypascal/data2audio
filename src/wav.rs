// use std::any::TypeId;
use std::collections::HashMap;
use std::ops::{ShlAssign, ShrAssign};
use std::path::{Path, PathBuf};

use hound::{self, Sample, WavReader, WavSpec, WavWriter};
// use i24::I24;

use crate::cli::{Args, SampleFormat};

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
    T: Copy + Sample + ShlAssign + ShlAssign<i32> + ShrAssign + ShrAssign<i32> + 'static, // &TypeId::of<T>() requires 'static
{
    let fmt_num_bits = HashMap::<SampleFormat, u16>::from([
        (SampleFormat::Int8, 8),
        (SampleFormat::Int16, 16),
        (SampleFormat::Int24, 24),
        (SampleFormat::Int32, 32),
        (SampleFormat::Vox, 16),
    ]);

    // let out_bits: u16;
    // let out_format: SampleFormat;
    //
    // if let Some(format) = &args.out_format {
    //     out_bits = fmt_num_bits[format];
    //     out_format = *format;
    // } else {
    //     out_bits = fmt_num_bits[&args.format];
    //     out_format = args.format;
    // }

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: args.samplerate,
        // bits_per_sample: out_bits,
        bits_per_sample: fmt_num_bits[&args.format],
        sample_format: hound::SampleFormat::Int,
    };

    //writer
    let mut writer = WavWriter::create(path, spec)?;

    // let shift_amount = (fmt_num_bits[&args.format] as i32) - (fmt_num_bits[&out_format] as i32);
    // if shift_amount != 0 {
    //     if shift_amount > 0 {
    //         for sample in data {
    //             let mut out_sample = *sample;
    //             out_sample <<= shift_amount;
    //             match out_format {
    //                 SampleFormat::Int8 => {
    //                     writer.write_sample(*sample as i8)?;
    //                 }
    //             }
    //         }
    //     }
    // } else if shift_amount < 0 {
    //     for sample in data {
    //         let mut out_sample = *sample;
    //         out_sample >>= shift_amount.abs();
    //         writer.write_sample(*sample)?;
    //     }
    // } else {
    for sample in data {
        writer.write_sample(*sample)?;
    }
    // }

    writer.finalize()?;

    Ok(())
}
