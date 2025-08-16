use hound::{WavSpec, WavWriter};
use std::{env::var_os, fs::File, io::BufWriter, path::PathBuf};

pub struct WaveWriter {
    writer: WavWriter<BufWriter<File>>,
}

impl WaveWriter {
    pub fn new(sample_rate: u32) -> Self {
        // Writing in mono for debugging simplicity. Ensure no pan is set in the channel!
        let target_file: PathBuf = {
            let os_var = var_os("CARGO_MANIFEST_DIR").unwrap();
            {
                let dir: PathBuf = os_var.into();
                dir.join("target/output.wav")
            }
        };
        println!("Saving wav file to: {:?}", target_file);
        let wav_spec = WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        WaveWriter { writer: WavWriter::create(target_file, wav_spec).unwrap() }
    }

    pub fn push(&mut self, sample: i16) {
        self.writer.write_sample(i16::from(sample)).unwrap();
    }

    pub fn write_file(self) {
        self.writer.finalize().unwrap();
    }
}
