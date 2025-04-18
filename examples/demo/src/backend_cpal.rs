use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, Sender};
use tato::audio::AudioChip;
use crate::wave_writer::WaveWriter;

pub struct AudioBackend {
    pub tx: Sender<Vec<i16>>,
    pub wav_file: WaveWriter,
    samples_per_frame: usize,
    sample_rate: u32,
    _stream: cpal::Stream,
}

impl AudioBackend {
    pub fn new(target_fps: f64) -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device");
        let config = device.default_output_config().unwrap();
        let sample_rate = config.sample_rate().0;
        let samples_per_frame = ((sample_rate as f64 / target_fps) * 2.0) as usize + 100;

        println!("Audio sample rate: {}", sample_rate);
        println!("Samples per frame: {}", samples_per_frame);

        let (tx, rx): (Sender<Vec<i16>>, Receiver<Vec<i16>>) = mpsc::channel();
        let mut sample_queue = VecDeque::with_capacity(samples_per_frame * 2);

        // Set up audio file writing for debugging, check "wave_writer" mod.
        let wav_file = WaveWriter::new(sample_rate);

        let _stream = device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    while let Ok(samples) = rx.try_recv() {
                        sample_queue.extend(samples);
                    }

                    for sample_slot in data.iter_mut() {
                        if let Some(sample) = sample_queue.pop_front() {
                            *sample_slot = sample as f32 / 32768.0;
                        } else {
                            *sample_slot = 0.0;
                        }
                    }
                },
                |err| eprintln!("Audio error: {}", err),
                None,
            )
            .unwrap();

        _stream.play().unwrap();

        AudioBackend {
            tx,
            samples_per_frame,
            sample_rate,
            wav_file,
            _stream,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn init_audio(&mut self, audio: &mut AudioChip) {
        let mut startup_samples = Vec::with_capacity(self.samples_per_frame * 2);
        for _ in 0..self.samples_per_frame {
            let sample = audio.process_sample();
            startup_samples.push(sample.left);
            startup_samples.push(sample.right);
            self.wav_file.push(sample.left);
        }
        let _ = self.tx.send(startup_samples);
    }

    #[inline]
    pub fn process_frame(&mut self, audio: &mut AudioChip) {
        let mut frame_samples = Vec::with_capacity(self.samples_per_frame);
        for _ in 0..self.samples_per_frame {
            let sample = audio.process_sample();
            frame_samples.push(sample.left);
            frame_samples.push(sample.right);
            self.wav_file.push(sample.left);
        }
        let _ = self.tx.send(frame_samples);
    }
}
