# data2audio

Takes a folder of any type of file(s) and converts all files to audio (often called “databending”). You can specify input/output paths, minimum file size, and the sample format at the input (i.e., treat incoming files as 8-bit unsigned, 16-bit integer, etc.)

Filters out sub-audible frequencies and normalizes amplitudes before writing to .WAV at the output.

## Usage

This project uses Rust's [cargo](https://doc.rust-lang.org/book/ch01-03-hello-cargo.html) package manager. After [installing Rust](https://doc.rust-lang.org/book/ch01-01-installation.html#installation), you can run the command `cargo run` from the code folder.

The code will default to expecting your input file(s) and/or folder(s) to be in the `input` subfolder, and will write .WAV files to the `output` sub-folder. Here are the commands to change the default options:

- `-h, --help` show this help message and exit
- `-i, --input` subfolder in which to look for files to import (string; default "input")
- `-o, --output` subfolder in which to write .WAV files (string; default "output")
- `-m, --min` minimum file size to convert (in bytes) — small files (< 1MB) are often less useful (int; default 0)
- `-s, --samplerate` sample rate at which to convert the incoming files to .WAV (int; default 44100)
- `-f, --format` sample format in which to read the files (string: options are 'uint8', 'int16', 'int24', 'int32', and 'vox'; default 'int16')
- `-F, --filter` whether to apply a 20 Hz low-cut filter to remove sub-audible frequencies (bool; default true)
- `-g, --gain` gain in decibels to apply before filtering (float; default -8.0)
  - When cutting out sub-audible frequencies, the peak-to-peak amplitude often increases. This setting is to compensate for that and avoid clipping. You may want to set this to 0 if not filtering.

### Usage Examples

- Note the extra two dashes (`--`) between `cargo run` and the command-line options. This sends your options to the running program, rather than to cargo.

- Only accept files 1MB or larger; read as unsigned 8-bit integer values

```sh
cargo run -- -m 1000000 -f 'uint8'
```

- Read files from the `data` subfolder and output them to the code folder, rather than a subfolder

```sh
cargo run -- -i "data" -o "."
```
