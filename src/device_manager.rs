use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use crate::events::EventType;
use crossbeam_channel::Sender;
use midir::{MidiInput, MidiInputPort};


const DEFAULT_MIDI_PORT: usize = 0;
const RUN_LOOP_SLEEP_DURATION_IN_MILLISECONDS: u64 = 100;
const MIDI_INPUT_CLIENT_NAME: &str = "Accidental Synth Input";
const NO_MIDI_INPUT_PORT_ERROR: &str = "No MIDI input ports found. Use the Arpeggiator";

pub struct DeviceManager {
    ui_sender: Sender<EventType>,
    midi_sender: Sender<EventType>,
    midi_input_ports: Arc<Mutex<Vec<String>>>,
    default_midi_input_port: Option<MidiInputPort>,
}


impl DeviceManager {
    pub fn new(ui_sender: Sender<EventType>, midi_sender: Sender<EventType>) -> Self {

        let mut default_midi_input_port = None;

        if let Ok(midi_input) = MidiInput::new(MIDI_INPUT_CLIENT_NAME) {
            default_midi_input_port = midi_input.ports().get(DEFAULT_MIDI_PORT).cloned();
        }

        Self {
            ui_sender,
            midi_sender,
            midi_input_ports: Arc::new(Mutex::new(Vec::new())),
            default_midi_input_port,
        }
    }


    pub fn run(&mut self, ui_sender: Sender<EventType>,) -> Result<(), Box<dyn Error>> {

        let midi_input_ports_arc = self.midi_input_ports.clone();
        let midi_input = MidiInput::new(MIDI_INPUT_CLIENT_NAME)?;

                loop {
                    let in_ports = midi_input.ports();
                    let default_midi_in_port = in_ports.get(DEFAULT_MIDI_PORT).cloned();
                    let midi_in_ports: Vec<String> = in_ports.iter().filter_map(|port| midi_input.port_name(port).ok()).collect();

                    let mut midi_input_ports  = midi_input_ports_arc.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                    if default_midi_in_port.is_some() && midi_in_ports != *midi_input_ports {
                        if let Err(err) = ui_sender.send(EventType::UpdateMidiPortList(midi_in_ports.clone())) {
                            eprintln!("Error sending event: {}", err);
                        }
                        *midi_input_ports = midi_in_ports;
                    }

                    sleep(Duration::from_millis(RUN_LOOP_SLEEP_DURATION_IN_MILLISECONDS));
                }
        }


    pub fn get_default_midi_input_port(&mut self) -> Option<MidiInputPort> {
        self.default_midi_input_port.clone()
    }

    pub fn get_midi_device_list(&mut self) -> Vec<String> {
        let midi_input_ports = self.midi_input_ports.lock().unwrap_or_else(|poisoned| poisoned
            .into_inner());
        midi_input_ports.clone()
    }


}