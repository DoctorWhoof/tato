use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, Sender};
use tato::audio::AudioChip;
use crate::WaveWriter;

pub struct AudioBackend {
    pub tx: Sender<Vec<i16>>,
    pub wav_file: Option<WaveWriter>,
    samples_per_frame: usize,
    sample_rate: u32,
    buffer_status_rx: Receiver<usize>, // Add feedback channel
    _stream: cpal::Stream,
}

impl AudioBackend {
    pub fn new(target_fps: f64) -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device");
        let config = device.default_output_config().unwrap();
        let sample_rate = config.sample_rate().0;

        // Calculate exact sample count (no +100 buffer)
        let exact_samples_per_frame = (sample_rate as f64 / target_fps) as usize;

        println!("Audio sample rate: {}", sample_rate);
        println!("Samples per frame: {}", exact_samples_per_frame);

        let (tx, rx): (Sender<Vec<i16>>, Receiver<Vec<i16>>) = mpsc::channel();
        // Create channel for buffer status feedback
        let (buffer_status_tx, buffer_status_rx) = mpsc::channel();

        let mut sample_queue = VecDeque::with_capacity(exact_samples_per_frame * 4);
        let _stream = device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Request more samples if needed
                    while let Ok(samples) = rx.try_recv() {
                        for value in samples {
                            sample_queue.push_back(value);
                        }
                    }

                    // Fill the output buffer
                    for sample_slot in data.iter_mut() {
                        if let Some(sample) = sample_queue.pop_front() {
                            *sample_slot = sample as f32 / 32768.0;
                        } else {
                            *sample_slot = 0.0;
                        }
                    }

                    // Report buffer status about every frame
                    let _ = buffer_status_tx.send(sample_queue.len());
                },
                |err| eprintln!("Audio error: {}", err),
                None,
            )
            .unwrap();

        _stream.play().unwrap();

        let wav_file = None;
        // let wav_file = Some(WaveWriter::new(sample_rate));

        AudioBackend {
            tx,
            samples_per_frame: exact_samples_per_frame,
            sample_rate,
            buffer_status_rx,
            wav_file,
            _stream,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn init_audio(&mut self, audio: &mut AudioChip) {
        let mut startup_samples = Vec::with_capacity(self.samples_per_frame);
        for _ in 0..self.samples_per_frame {
            let sample = audio.process_sample();
            startup_samples.push(sample.left);
            startup_samples.push(sample.right);
            if let Some(wav_file) = &mut self.wav_file {
                wav_file.push(sample.left);
            }
            // self.wav_file.push(sample.right);
        }
        let _ = self.tx.send(startup_samples);
    }

    #[inline]
    pub fn process_frame(&mut self, audio: &mut AudioChip) {
        // Check current buffer status
        let mut additional_samples = 0;

        // Try to get the latest buffer status
        while let Ok(buffer_size) = self.buffer_status_rx.try_recv() {
            // If buffer is low, add more samples
            // Target maintaining ~2 frames worth of samples in the buffer
            let target_buffer = self.samples_per_frame * 2;
            if buffer_size < target_buffer {
                additional_samples = target_buffer - buffer_size;
            }
        }

        // Generate samples for this frame, plus any additional if needed
        let total_samples = self.samples_per_frame + additional_samples/2; // Divide by 2 for stereo
        let mut frame_samples = Vec::with_capacity(total_samples * 2);

        for _ in 0..total_samples {
            let sample = audio.process_sample();
            frame_samples.push(sample.left);
            frame_samples.push(sample.right);
            if let Some(wav_file) = &mut self.wav_file {
                wav_file.push(sample.left);
            }
        }

        let _ = self.tx.send(frame_samples);
    }

    pub fn write_wav_file(self){
        if let Some(wav_file) = self.wav_file {
            wav_file.write_file();
        }
    }
}
