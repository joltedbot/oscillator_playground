mod device_manager;
mod events;
mod midi;
mod synth;
mod ui;

use crate::device_manager::DeviceManager;
use crate::midi::Midi;
use crate::synth::Synth;
use crate::synth::device::AudioDevice;
use crate::ui::UI;
use slint::ComponentHandle;
use std::thread;

slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let application = ApplicationWindow::new()?;

    let events = events::Events::new();

    let synth_sender = events.get_synth_sender();
    let synth_receiver = events.get_synth_receiver();
    let ui_sender = events.get_ui_sender();
    let ui_receiver = events.get_ui_receiver();
    let midi_receiver = events.get_midi_receiver();
    let midi_sender = events.get_midi_sender();

    let mut ui = UI::new(
        application.as_weak(),
        synth_sender.clone(),
        midi_sender.clone(),
        ui_sender.clone(),
    )
    .expect("Could not create UI");

    ui.create_ui_callbacks();

    thread::spawn(move || {
        ui.run(ui_receiver.clone());
    });

    let mut device_manager = DeviceManager::new();
    let default_midi_input_port = device_manager.get_default_midi_input_port();

    thread::spawn(move || {
        device_manager
            .run(ui_sender.clone())
            .expect("Could not run device manager");
    });

    let mut midi = Midi::new(default_midi_input_port);
    midi.run(synth_sender.clone(), midi_receiver.clone());

    // Initialize the default audio output device for your system
    let audio_device = AudioDevice::new();

    thread::spawn(|| {
        let mut synth = Synth::new(audio_device);
        synth.run(synth_receiver);
    });

    application.run()
}
