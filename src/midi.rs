use crate::events::EventType;
use crossbeam_channel::Sender;
use midir::{Ignore, MidiInput};
use std::io::stdin;
use std::thread;

const DEFAULT_MIDI_PORT: usize = 0;
const MIDI_STATUS_BYTE_INDEX: usize = 0;
const MIDI_NOTE_NUMBER_BYTE_INDEX: usize = 1;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum MessageType {
    NoteOff,
    NoteOn,
    PolyphonicKeyPressure,
    ControlChange,
    ProgramChange,
    ChannelPressure,
    PitchBend,
    Unknown,
}

pub struct Midi {}

impl Midi {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self, synth_sender: Sender<EventType>) {
        let mut input = String::new();

        thread::spawn(move || {
            let mut midi_in = MidiInput::new("Accidental Synthesizer").unwrap();
            midi_in.ignore(Ignore::SysexAndTime);
            midi_in.ignore(Ignore::ActiveSense);

            let in_ports = midi_in.ports();
            let in_port = match in_ports.get(DEFAULT_MIDI_PORT) {
                Some(port) => port,
                None => {
                    println!("No MIDI input ports found. Use the Arpeggiator");
                    return;
                }
            };

            println!("\nOpening connection");
            let in_port_name = midi_in.port_name(in_port).unwrap();

            let _conn_in = midi_in
                .connect(
                    in_port,
                    "midir-read-input",
                    move |_, message, _| {
                        let message_type =
                            get_midi_message_type_from_status_byte(message[MIDI_STATUS_BYTE_INDEX]);
                        match message_type {
                            MessageType::NoteOn => {
                                if let Err(error) = synth_sender.send(
                                    EventType::MidiNoteOn(message[MIDI_NOTE_NUMBER_BYTE_INDEX])
                                        .clone(),
                                ) {
                                    eprintln!("Error sending event: {error}",);
                                }
                            }
                            MessageType::NoteOff => {
                                if let Err(error) = synth_sender.send(
                                    EventType::MidiNoteOff(message[MIDI_NOTE_NUMBER_BYTE_INDEX])
                                        .clone(),
                                ) {
                                    eprintln!("Error sending event: {error}",);
                                }
                            }
                            _ => {}
                        }
                    },
                    (),
                )
                .unwrap();

            println!("Connection open, reading input from '{}' ...", in_port_name);

            input.clear();
            stdin().read_line(&mut input).unwrap();
        });

        println!("Midi Running");
    }
}

fn get_midi_message_type_from_status_byte(status: u8) -> MessageType {
    let status_type = status & 0xF0;
    let _status_channel = status & 0x0F;
    match status_type {
        0x80 => MessageType::NoteOff,
        0x90 => MessageType::NoteOn,
        0xA0 => MessageType::PolyphonicKeyPressure,
        0xB0 => MessageType::ControlChange,
        0xC0 => MessageType::ProgramChange,
        0xD0 => MessageType::ChannelPressure,
        0xE0 => MessageType::PitchBend,
        _ => MessageType::Unknown,
    }
}
