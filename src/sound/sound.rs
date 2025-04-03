use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{self, Sender};


pub struct SoundEngine {
    stream: Option<Stream>,
    config: StreamConfig,
    active_sounds: Arc<Mutex<Vec<(f32, f32, usize)>>>,
}

impl SoundEngine {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("Failed to get default output device");
        let config = device.default_output_config().expect("Failed to get default output config").config();

        Self {
            stream: None,
            config,
            active_sounds: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn init(&mut self) {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("Failed to get default output device");
        let config = self.config.clone();
        let active_sounds = self.active_sounds.clone();

        //ok I feel like this whole thins is just... overly complicated for what it is
        // idk though like erm... not my problem well it is but like yeah...
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut sounds = active_sounds.lock().unwrap();
                data.iter_mut().for_each(|sample| *sample = 0.0);
                
                sounds.retain_mut(|(freq, phase, remaining_samples)| {
                    if *remaining_samples == 0 {
                        return false;
                    }
                    for sample in data.iter_mut() {
                        *sample += (2.0 * std::f32::consts::PI * *freq * *phase as f32 / config.sample_rate.0 as f32).sin() * 0.3; // the 0.3 is like the amplitude so like maybe change that later or have it be controllable
                        *phase += 1.0;
                        *remaining_samples -= 1;
                    }
                    true
                });
                
                // Prevent clipping
                data.iter_mut().for_each(|sample| *sample = sample.clamp(-1.0, 1.0));
            },
            |err| eprintln!("Stream error: {}", err),
            None,
        ).expect("Failed to build output stream");

        stream.play().expect("Failed to start stream");
        self.stream = Some(stream);
    }

    pub fn play_sequence(&self, sequence: &Vec<(u8, f32, Duration)>) {
        let sample_rate = self.config.sample_rate.0 as f32;
        let mut sounds = self.active_sounds.lock().unwrap();
        
        for &(_, freq, duration) in sequence {
            let sample_count = (sample_rate * duration.as_secs_f32()) as usize;
            sounds.push((freq, 0.0, sample_count));
        }
    }
}

pub struct SoundManager {
    sender: Sender<(u8, f32, Duration)>,
}

impl SoundManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<(u8, f32, Duration)>();

        thread::spawn(move || {
            let mut sound_engine = SoundEngine::new();
            sound_engine.init();

            while let Ok((note, freq, duration)) = rx.recv() {
                println!("Playing note {} at {:.2} Hz", note, freq);
                sound_engine.play_sequence(&vec![(note, freq, duration)]);
            }
        });

        Self { sender: tx }
    }

    pub fn play_note(&self, note: u8, freq: f32, duration: Duration) {//this should probably be mut... but eeh
        self.sender.send((note, freq, duration)).expect("failed to send note to thread");
    }
}