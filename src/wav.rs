use hound::{self, Sample, WavReader, WavSpec, WavWriter};
use std::path::{Path, PathBuf};

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
pub fn write_file_as_wav<T>(data: &[T], path: &PathBuf, spec: &WavSpec) -> Result<(), hound::Error>
where
    T: Copy + Sample,
{
    //writer
    let mut writer = WavWriter::create(path, *spec)?;
    for sample in data {
        writer.write_sample(*sample)?;
    }
    writer.finalize()?;

    Ok(())
}
