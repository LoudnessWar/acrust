use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{f32::consts::PI, time::Duration};

fn play() {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device found");
    let config = device.default_output_config().unwrap();

    let sample_rate = config.sample_rate().0 as f32;
    let mut phase = 0.0;
    let freq = 440.0; // A4 note

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for sample in data.iter_mut() {
                *sample = (phase * 2.0 * PI).sin();
                phase += freq / sample_rate;
                if phase > 1.0 { phase -= 1.0; }
            }
        },
        |err| eprintln!("Stream error: {}", err), Some(Duration::new(5, 0))
    ).unwrap();

    stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(5)); // Play sound for 5 seconds
}