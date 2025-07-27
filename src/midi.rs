use std::error::Error;
use std::sync::{Arc, Mutex};
use crate::events::EventType;
use crossbeam_channel::{Receiver, Sender};
use midir::{MidiIO, MidiInput, MidiInputConnection, MidiInputPort};
use std::thread;

const DEFAULT_MIDI_PORT: usize = 0;
const MIDI_STATUS_BYTE_INDEX: usize = 0;
const MIDI_NOTE_NUMBER_BYTE_INDEX: usize = 1;
const MIDI_INPUT_CLIENT_NAME: &str = "Accidental Synth Input";

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

pub struct Midi {
    input_connection: Arc<Mutex<Option<MidiInputConnection<()>>>>,
    input_port: Arc<Mutex<Option<MidiInputPort>>>,
}

impl Midi {
    pub fn new(default_input_port: Option<MidiInputPort>) -> Self {
        Self {
            input_connection: Arc::new(Mutex::new(None)),
            input_port: Arc::new(Mutex::new(default_input_port)),
        }
    }

    pub fn run(&mut self, synth_sender: Sender<EventType>, midi_receiver: Receiver<EventType>) {

        let input_connection_arc = self.input_connection.clone();
        let input_port_arc = self.input_port.clone();




        thread::spawn(move || {

            let mut input_connection = input_connection_arc.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let mut input_port = input_port_arc.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

            let input_port_option = input_port.to_owned();

            if let Some(input_port) = input_port_option {
                *input_connection = match create_midi_listener(synth_sender.clone(), input_port) {
                    Ok(connection) => {
                        Some(connection)
                    }
                    Err(error) => {
                        eprintln!("Error creating MIDI listener: {error}", );
                        None
                    }
                };
            }

            while let Ok(event) = midi_receiver.recv() {
                match event {
                    EventType::UpdateMidiPort(port_index) => {

                        let port_result =   MidiInput::new(MIDI_INPUT_CLIENT_NAME);

                        if let Ok(midi_input) = port_result {
                            let input_ports= midi_input.ports();
                            let input_port = input_ports.get(port_index as usize).cloned();
                            if let Some(port) = input_port {
                                *input_connection = create_midi_listener(synth_sender.clone(), port).ok();
                            }

                        } else {
                            eprintln!("Error creating MIDI input");
                        }

                    }
                    _ => {}
                }
            }

        });

    }
}

fn create_midi_listener(synth_sender: Sender<EventType>, in_port: MidiInputPort) -> Result<MidiInputConnection<()>, Box<dyn Error>> {

    let midi_in = MidiInput::new(MIDI_INPUT_CLIENT_NAME)?;

   midi_in
        .connect(
            &in_port,
            "midir-read-input",
            move |_, message, _| {


                let message_type = get_midi_message_type_from_status_byte(message[MIDI_STATUS_BYTE_INDEX]);

                match message_type {
                    MessageType::NoteOn => {
                        if let Err(error) = synth_sender.send(
                            EventType::MidiNoteOn(message[MIDI_NOTE_NUMBER_BYTE_INDEX])
                                .clone(),
                        ) {
                            eprintln!("Error sending event: {error}", );
                        }
                    }
                    MessageType::NoteOff => {
                        if let Err(error) = synth_sender.send(
                            EventType::MidiNoteOff(message[MIDI_NOTE_NUMBER_BYTE_INDEX])
                                .clone(),
                        ) {
                            eprintln!("Error sending event: {error}", );
                        }
                    }
                    _ => {}
                }
            },
            (),
        ).map_err(|error| Box::new(error) as Box<dyn Error>)

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


