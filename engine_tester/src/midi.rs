use midir::{MidiInput, Ignore};
use midly::{Smf, TrackEvent, TrackEventKind, MidiMessage, Format, Timing, Header};
use std::{sync::mpsc::Sender, time::Instant, fs::File, io::Write};

pub struct MidiHandler;

impl MidiHandler {
    pub fn new() -> Self {
        MidiHandler
    }
    
    pub fn start_listening(&self, tx: Sender<(u8, Instant)>) {
        let mut midi_in = MidiInput::new("MIDI Receiver").expect("Failed to create MIDI input");
        midi_in.ignore(Ignore::None);
        let in_ports = midi_in.ports();
        if in_ports.is_empty() { panic!("No MIDI input ports available"); }
        let in_port = &in_ports[0];
        println!("Using MIDI input: {}", midi_in.port_name(in_port).unwrap());
        
        midi_in.connect(in_port, "midi-in", move |_, message, _| {
            if message.len() > 2 && message[0] & 0xF0 == 0x90 && message[2] > 0 {
                let note = message[1];
                tx.send((note, Instant::now())).unwrap();
            }
        }, ()).expect("Failed to open MIDI connection");
    }
    
    pub fn midi_to_freq(&self, note: u8) -> f32 {
        440.0 * (2.0_f32).powf((note as f32 - 69.0) / 12.0)
    }
    
    pub fn save_to_midi(&self, sequence: &Vec<(u8, f32, std::time::Duration)>) {
        let mut smf = Smf::new(Header::new(Format::SingleTrack, Timing::Metrical((480.into()))));
        let mut track = vec![];
        for (note, _, timestamp) in sequence {
            track.push(TrackEvent {
                delta: (timestamp.as_millis() as u32).into(),
                kind: TrackEventKind::Midi { channel: 0.into(), message: MidiMessage::NoteOn { key: (*note).into(), vel: 64.into() }},
            });
        }
        smf.tracks.push(track);
        let mut file = File::create("output.mid").unwrap();
        smf.write_std(&mut file).unwrap();
        println!("Saved sequence to output.mid");
    }
}
