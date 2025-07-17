mod events;
mod synth;
mod ui;

use crate::synth::Synth;
use crate::synth::device::AudioDevice;
use crate::ui::UI;
use slint::ComponentHandle;
use std::sync::{Arc, Mutex};
use std::thread;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    // Initialize Slint Application
    let application = AppWindow::new()?;

    let events = events::Events::new();

    let synth_sender = events.get_synth_sender();
    let syn_receiver = events.get_synth_receiver();

    // Initialize the default audio output device for your system
    let audio_device = AudioDevice::new();

    thread::spawn(|| {
        let mut synth = Synth::new(audio_device);
        synth.run(syn_receiver);
    });

    let mut ui = UI::new(Arc::new(Mutex::new(application.as_weak())), synth_sender)
        .expect("Could not create UI");

    ui.create_ui_callbacks();

    application.run()
}
