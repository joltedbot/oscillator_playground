use crate::events::EventType;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, default_host};
use crossbeam_channel::Sender;
use midir::{MidiInput, MidiInputPort};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

const DEFAULT_MIDI_PORT: usize = 0;
const RUN_LOOP_SLEEP_DURATION_IN_MILLISECONDS: u64 = 400;
const MIDI_INPUT_CLIENT_NAME: &str = "Accidental Synth Input";

#[derive(Clone, Default, Debug, PartialEq)]
pub struct DeviceList {
    pub devices: Vec<String>,
    pub channels: Vec<Vec<String>>,
}

#[derive(Clone, Default)]
pub struct DeviceManager {
    midi_input_ports: Arc<Mutex<Vec<String>>>,
    default_midi_input_port: Option<MidiInputPort>,
    output_devices: Arc<Mutex<DeviceList>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        let mut default_midi_input_port = None;

        if let Ok(midi_input) = MidiInput::new(MIDI_INPUT_CLIENT_NAME) {
            default_midi_input_port = midi_input.ports().get(DEFAULT_MIDI_PORT).cloned();
        }

        Self {
            midi_input_ports: Arc::new(Mutex::new(Vec::new())),
            default_midi_input_port,
            ..Default::default()
        }
    }

    pub fn run(&mut self, ui_sender: Sender<EventType>) -> Result<(), Box<dyn Error>> {
        let mut output_devices_arc = self.output_devices.clone();
        let mut midi_input_ports_arc = self.midi_input_ports.clone();
        let midi_input = MidiInput::new(MIDI_INPUT_CLIENT_NAME)?;

        loop {
            update_midi_input_port_if_changed(&ui_sender, &mut midi_input_ports_arc, &midi_input);
            update_audio_output_device_if_changed(&ui_sender, &mut output_devices_arc)?;

            sleep(Duration::from_millis(
                RUN_LOOP_SLEEP_DURATION_IN_MILLISECONDS,
            ));
        }
    }

    pub fn get_default_midi_input_port(&mut self) -> Option<MidiInputPort> {
        self.default_midi_input_port.clone()
    }
}

fn update_midi_input_port_if_changed(
    ui_sender: &Sender<EventType>,
    midi_input_ports_arc: &mut Arc<Mutex<Vec<String>>>,
    midi_input: &MidiInput,
) {
    let in_ports = midi_input.ports();
    let default_midi_in_port = in_ports.get(DEFAULT_MIDI_PORT).cloned();
    let midi_in_ports: Vec<String> = in_ports
        .iter()
        .filter_map(|port| midi_input.port_name(port).ok())
        .collect();

    let mut midi_input_ports = midi_input_ports_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if default_midi_in_port.is_some() && midi_in_ports != *midi_input_ports {
        if let Err(err) = ui_sender.send(EventType::UpdateMidiPortList(midi_in_ports.clone())) {
            eprintln!("Error sending event: {}", err);
        }

        *midi_input_ports = midi_in_ports;
    }
}

fn update_audio_output_device_if_changed(
    ui_sender: &Sender<EventType>,
    current_output_devices_arc: &mut Arc<Mutex<DeviceList>>,
) -> Result<(), Box<dyn Error>> {
    let output_devices = get_output_device_list_from_host()?;
    let mut current_output_devices = current_output_devices_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if output_devices != *current_output_devices {
        *current_output_devices = output_devices.clone();
        ui_sender.send(EventType::UpdateOutputDeviceList(output_devices))?;
    }

    Ok(())
}

fn get_output_device_list_from_host() -> Result<DeviceList, Box<dyn Error>> {
    let mut output_devices: Vec<String> = Vec::new();
    let mut output_channels: Vec<Vec<String>> = Vec::new();

    let host = default_host();

    host.output_devices()?.for_each(|device| {
        if let Ok(name) = device.name() {
            output_devices.push(name);
            output_channels.push(get_channel_list_from_output_device(&device));
        }
    });

    Ok(DeviceList {
        devices: output_devices,
        channels: output_channels,
    })
}

fn get_channel_list_from_output_device(output_device: &Device) -> Vec<String> {
    if let Ok(config) = output_device.default_output_config() {
        let number_of_output_channels = config.channels();
        let channels = (1..=number_of_output_channels)
            .map(|i| i.to_string())
            .collect();

        return channels;
    };

    Vec::new()
}
