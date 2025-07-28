use crate::events::EventType;
use crossbeam_channel::{Receiver, Sender};
use midir::{MidiInput, MidiInputConnection, MidiInputPort};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;

const MIDI_STATUS_BYTE_INDEX: usize = 0;
const MIDI_NOTE_NUMBER_BYTE_INDEX: usize = 1;
const MIDI_CHANNEL_FOR_OMNI: i32 = 0;
const MIDI_CHANNEL_OFFSET_USER_VS_INDEX: i32 = 1;
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
    input_channel: Arc<Mutex<i32>>,
}

impl Midi {
    pub fn new(default_input_port: Option<MidiInputPort>) -> Self {
        Self {
            input_connection: Arc::new(Mutex::new(None)),
            input_port: Arc::new(Mutex::new(default_input_port)),
            input_channel: Arc::new(Mutex::new(MIDI_CHANNEL_FOR_OMNI)),
        }
    }

    pub fn run(&mut self, synth_sender: Sender<EventType>, midi_receiver: Receiver<EventType>) {
        let input_connection_arc = self.input_connection.clone();
        let input_port_arc = self.input_port.clone();
        let midi_channel_arc = self.input_channel.clone();

        let mut input_connection = input_connection_arc
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let input_port = input_port_arc
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let input_port_option = input_port.to_owned();

        if let Some(input_port) = input_port_option {
            *input_connection = create_new_midi_listener(
                synth_sender.clone(),
                input_port,
                midi_channel_arc.clone(),
            )
            .ok();
        }

        let input_connection_thread_arc = self.input_connection.clone();

        thread::spawn(move || {
            while let Ok(event) = midi_receiver.recv() {
                match event {
                    EventType::UpdateMidiPort(port_index) => {
                        let mut input_connection = input_connection_thread_arc
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());

                        if let Some(port) = get_midi_input_port_from_port_index(port_index) {
                            *input_connection = create_new_midi_listener(
                                synth_sender.clone(),
                                port,
                                midi_channel_arc.clone(),
                            )
                            .ok();
                        }
                    }
                    EventType::UpdateMidiChannel(channel) => {
                        let mut midi_channel = midi_channel_arc
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        *midi_channel = channel;
                    }
                    _ => {}
                }
            }
        });
    }
}

fn create_new_midi_listener(
    synth_sender: Sender<EventType>,
    in_port: MidiInputPort,
    midi_channel: Arc<Mutex<i32>>,
) -> Result<MidiInputConnection<()>, Box<dyn Error>> {
    let midi_in = MidiInput::new(MIDI_INPUT_CLIENT_NAME)?;
    let midi_channel_arc = midi_channel.clone();

    midi_in
        .connect(
            &in_port,
            "midir-read-input",
            move |_, message, _| {
                let current_midi_channel = midi_channel_arc
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let message_channel =
                    get_midi_channel_type_from_status_byte(message[MIDI_STATUS_BYTE_INDEX]);

                if *current_midi_channel != MIDI_CHANNEL_FOR_OMNI
                    && *current_midi_channel != message_channel
                {
                    return;
                }

                let message_type =
                    get_midi_message_type_from_status_byte(message[MIDI_STATUS_BYTE_INDEX]);

                match message_type {
                    MessageType::NoteOn => {
                        if let Err(error) = synth_sender.send(
                            EventType::MidiNoteOn(message[MIDI_NOTE_NUMBER_BYTE_INDEX]).clone(),
                        ) {
                            eprintln!("Error sending event: {error}",);
                        }
                    }
                    MessageType::NoteOff => {
                        if let Err(error) = synth_sender.send(
                            EventType::MidiNoteOff(message[MIDI_NOTE_NUMBER_BYTE_INDEX]).clone(),
                        ) {
                            eprintln!("Error sending event: {error}",);
                        }
                    }
                    _ => {}
                }
            },
            (),
        )
        .map_err(|error| Box::new(error) as Box<dyn Error>)
}

fn get_midi_input_port_from_port_index(port_index: i32) -> Option<MidiInputPort> {
    MidiInput::new(MIDI_INPUT_CLIENT_NAME)
        .ok()
        .and_then(|midi_input| midi_input.ports().get(port_index as usize).cloned())
}

fn get_midi_message_type_from_status_byte(status: u8) -> MessageType {
    let status_type = status & 0xF0;
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

fn get_midi_channel_type_from_status_byte(status: u8) -> i32 {
    (status & 0x0F) as i32 + MIDI_CHANNEL_OFFSET_USER_VS_INDEX
}
