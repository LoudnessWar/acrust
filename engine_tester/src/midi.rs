//ok so I think midi is cool so thats why i do this
use midir::{MidiInput, Ignore};

pub struct midi_Handle{
    pub midi_in: MidiInput,
}

impl midi_Handle{
    pub fn new(){
        let mut midi_in = MidiInput::new("MIDI Receiver").expect("Failed to create MIDI input");
        
        midi_in.ignore(Ignore::None);

        let in_ports = midi_in.ports();
        if in_ports.is_empty() {
            panic!("No MIDI input ports available");
        }
        
        let in_port = &in_ports[0]; // Choose the first available MIDI port
        println!("Using MIDI input: {}", midi_in.port_name(in_port).unwrap());

        // Create a channel for handling MIDI events asynchronously
        let (tx, rx) = mpsc::channel();

        let _conn = midi_in.connect(in_port, "midi-in", move |_, message, _| {
            tx.send(message.to_vec()).unwrap();
        }, ()).expect("Failed to open MIDI connection");

        thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                println!("Received MIDI message: {:?}", msg);
            }
        });
    }
}