use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct SoundEngine {
    stream: Option<Stream>,
    config: StreamConfig,
    sample_buffer: Arc<Mutex<Vec<f32>>>,
}
//ok so the deal is with this right... we have 2 types of audio like in the ether
//we can seperate it into two types. there is like real time audio and then there is like
//just playing a .wav file or something. They will both require different like methods
//and such to impliment
//... that is all like this right now only has the real time I need to optimize it to
//be able to handle both so like thats a TODO
impl SoundEngine {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("Failed to get default output device");
        let config = device.default_output_config().expect("Failed to get default output config").config();

        Self {
            stream: None,
            config,
            sample_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn init(&mut self) {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("Failed to get default output device");
        let config = self.config.clone();
        let sample_buffer = self.sample_buffer.clone();
        
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut buffer = sample_buffer.lock().unwrap();
                for (i, sample) in data.iter_mut().enumerate() {
                    *sample = buffer.get(i).cloned().unwrap_or(0.0);
                }
                if buffer.len() >= data.len() {
                    buffer.drain(0..data.len());
                }
            },
            |err| eprintln!("Stream error: {}", err),
            None,
        ).expect("Failed to build output stream");

        stream.play().expect("Failed to start stream");
        self.stream = Some(stream);
    }

    pub fn play_sequence(&self, sequence: &Vec<(u8, f32, Duration)>) {
        let sample_rate = self.config.sample_rate.0 as f32;
        let mut buffer = self.sample_buffer.lock().unwrap();
        
        for &(_, freq, duration) in sequence {
            let sample_count = (sample_rate * duration.as_secs_f32()) as usize;
            for i in 0..sample_count {
                let sample = (2.0 * std::f32::consts::PI * freq * i as f32 / sample_rate).sin() * 0.3; // Apply volume scaling
                buffer.push(sample);
            }
        }
    }
    

    // pub fn play_sequence(&self, sequence: &Vec<(u8, f32, Duration)>) {
    //     if let Some(stream) = &self.stream {
    //         let sample_rate = self.config.sample_rate.0 as f32;

    //         let mut samples = vec![0.0; (sample_rate as usize * 2)];
    //         for (note, freq, timestamp) in sequence {
    //             let phase_step = 2.0 * std::f32::consts::PI * *freq / sample_rate;
    //             let mut phase: f32 = 0.0;
    //             for sample in samples.iter_mut() {
    //                 *sample += (phase.sin() * 0.3) as f32;
    //                 phase += phase_step;
    //             }
    //             thread::sleep(*timestamp);
    //         }
            
    //         let sample_buffer = Arc::new(Mutex::new(samples));
    //         //let err_fn = |err| eprintln!("Error in sound stream: {}", err);
                
    //         stream.play().expect("Failed to start audio playback");
    //     }
    // }

    // pub fn play_sequence(&self, sequence: &Vec<(u8, f32, Duration)>) {
    //     let host = cpal::default_host();
    //     let device = host.default_output_device().expect("No audio output device found");
    //     let config = device.default_output_config().expect("Failed to get default output config");
    //     let sample_rate = config.sample_rate().0 as f32;
    //     let channels = config.channels() as usize;
        
    //     let mut samples = vec![0.0; (sample_rate as usize * 2)];
    //     for (note, freq, timestamp) in sequence {
    //         let phase_step = 2.0 * std::f32::consts::PI * *freq / sample_rate;
    //         let mut phase: f32 = 0.0;
    //         for sample in samples.iter_mut() {
    //             *sample += (phase.sin() * 0.3) as f32;
    //             phase += phase_step;
    //         }
    //         thread::sleep(*timestamp);
    //     }
        
    //     let sample_buffer = Arc::new(Mutex::new(samples));
    //     let err_fn = |err| eprintln!("Error in sound stream: {}", err);
        
    //     let stream = device.build_output_stream(
    //         &config.into(),
    //         move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
    //             let mut buffer = sample_buffer.lock().unwrap();
    //             for (i, sample) in data.iter_mut().enumerate() {
    //                 *sample = buffer[i % buffer.len()];
    //             }
    //         },
    //         err_fn,
    //         None,
    //     ).expect("Failed to build audio stream");
        
    //     stream.play().expect("Failed to start audio playback");
    //     thread::sleep(Duration::from_secs(2));
    // }

    // pub fn play_sound_file(&self, file_path: &str) {
    //     let mut file = File::open(file_path).expect("Failed to open file");
    //     let mut buffer = Vec::new();
    //     file.read_to_end(&mut buffer).expect("Failed to read file");
        
    //     if let Some(stream) = &self.stream {
    //         let sample_iter = buffer.iter().map(|&b| b as f32 / 255.0); // Convert to float samples
            
    //         stream.play().expect("Failed to start audio playback");
    //         for (sample, data_sample) in sample_iter.zip(buffer.iter()) {
    //             // Example of writing to buffer (this should be adapted to cpal's specific format)
    //             *data_sample = (sample * 2.0 - 1.0) as u8;
    //         }
    //     }
    // }
}
