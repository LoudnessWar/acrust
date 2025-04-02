use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct SoundEngine;

impl SoundEngine {
    pub fn new() -> Self {
        SoundEngine
    }
    
    pub fn play_sequence(&self, sequence: &Vec<(u8, f32, Duration)>) {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No audio output device found");
        let config = device.default_output_config().expect("Failed to get default output config");
        let sample_rate = config.sample_rate().0 as f32;
        let channels = config.channels() as usize;
        
        let mut samples = vec![0.0; (sample_rate as usize * 2)];
        for (note, freq, timestamp) in sequence {
            let phase_step = 2.0 * std::f32::consts::PI * *freq / sample_rate;
            let mut phase: f32 = 0.0;
            for sample in samples.iter_mut() {
                *sample += (phase.sin() * 0.3) as f32;
                phase += phase_step;
            }
            thread::sleep(*timestamp);
        }
        
        let sample_buffer = Arc::new(Mutex::new(samples));
        let err_fn = |err| eprintln!("Error in sound stream: {}", err);
        
        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut buffer = sample_buffer.lock().unwrap();
                for (i, sample) in data.iter_mut().enumerate() {
                    *sample = buffer[i % buffer.len()];
                }
            },
            err_fn,
            None,
        ).expect("Failed to build audio stream");
        
        stream.play().expect("Failed to start audio playback");
        thread::sleep(Duration::from_secs(2));
    }
}
