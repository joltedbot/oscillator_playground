use super::AppWindow;
use crate::events::EventType;
use crossbeam_channel::{Receiver, Sender};
use slint::Weak;
use std::error::Error;
use std::process::exit;
use std::sync::{Arc, Mutex};

pub struct UI {
    pub ui: Weak<AppWindow>,
    synth_sender: Sender<EventType>,
}

impl UI {
    pub fn new(
        ui_mutex: Arc<Mutex<Weak<AppWindow>>>,
        synth_sender: Sender<EventType>,
    ) -> Result<Self, Box<dyn Error>> {
        let ui_weak = ui_mutex
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let ui = Self {
            ui: ui_weak.clone(),
            synth_sender,
        };

        Ok(ui)
    }

    pub fn run(&mut self, ui_receiver: Receiver<EventType>) -> Result<(), Box<dyn Error>> {
        loop {
            if let Ok(event) = ui_receiver.recv() {
                match event {
                    EventType::Exit => {
                        break;
                    }
                    _ => (),
                }
            }
        }

        Ok(())
    }

    pub fn create_ui_callbacks(&mut self) {
        self.on_start_button_pressed_callback();
        self.on_wave_shape1_selected_callback();
        self.on_wave_shape2_selected_callback();
        self.on_wave_shape3_selected_callback();
        self.on_sub_shape_selected_callback();
        self.on_wave_level1_selected_callback();
        self.on_wave_level2_selected_callback();
        self.on_wave_level3_selected_callback();
        self.on_sub_level_selected_callback();
        self.on_wave_detune_value_changed();
        self.on_wave_detune_stage_changed();
        self.on_output_level_value_changed();
    }

    fn get_ui_reference_from_ui_weak(&mut self) -> AppWindow {
        let ui_weak = self.ui.clone();

        match ui_weak.upgrade() {
            Some(ui) => ui,
            None => {
                eprintln!("Could not upgrade ui mutex");
                exit(1);
            }
        }
    }

    fn on_wave_shape1_selected_callback(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_shape1_selected(move |shape| {
            if let Err(error) = synth_sender.send(EventType::UpdateOscillator1Shape(shape)) {
                eprintln!("Error sending event: {}", error);
            };
        });
    }

    fn on_wave_shape2_selected_callback(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_shape2_selected(move |shape| {
            if let Err(error) = synth_sender.send(EventType::UpdateOscillator2Shape(shape)) {
                eprintln!("Error sending event: {}", error);
            };
        });
    }

    fn on_wave_shape3_selected_callback(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_shape3_selected(move |shape| {
            if let Err(error) = synth_sender.send(EventType::UpdateOscillator3Shape(shape)) {
                eprintln!("Error sending event: {}", error);
            };
        });
    }

    fn on_sub_shape_selected_callback(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_sub_shape_selected(move |shape| {
            if let Err(error) = synth_sender.send(EventType::UpdateSubOscillatorShape(shape)) {
                eprintln!("Error sending event: {}", error);
            };
        });
    }

    fn on_wave_level1_selected_callback(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_level1_selected(move |level| {
            if let Err(error) = synth_sender.send(EventType::UpdateOscillator1Level(level)) {
                eprintln!("Error sending event: {}", error);
            };
        });
    }
    fn on_wave_level2_selected_callback(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_level2_selected(move |level| {
            if let Err(error) = synth_sender.send(EventType::UpdateOscillator2Level(level)) {
                eprintln!("Error sending event: {}", error);
            };
        });
    }
    fn on_wave_level3_selected_callback(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_level3_selected(move |level| {
            if let Err(error) = synth_sender.send(EventType::UpdateOscillator3Level(level)) {
                eprintln!("Error sending event: {}", error);
            };
        });
    }

    fn on_sub_level_selected_callback(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_sub_level_selected(move |level| {
            if let Err(error) = synth_sender.send(EventType::UpdateSubOscillatorLevel(level)) {
                eprintln!("Error sending event: {}", error);
            };
        });
    }

    fn on_start_button_pressed_callback(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_start_button_pressed(move |is_active| {
            let event_type = match is_active {
                true => EventType::Start,
                false => EventType::Stop,
            };

            if let Err(error) = synth_sender.send(event_type.clone()) {
                eprintln!("Error sending event: {}", error);
            };
        });
    }

    fn on_wave_detune_value_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_detune_value_changed(move |detune_amount| {
            if let Err(error) =
                synth_sender.send(EventType::UpdateOscillatorDetuneValue(detune_amount))
            {
                eprintln!("Error sending event: {}", error);
            };
        });
    }

    fn on_wave_detune_stage_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_detune_stage_changed(move |is_active, detune_amount| {
            if let Err(error) = synth_sender
                .send(EventType::UpdateOscillatorDetuneActive(is_active, detune_amount).clone())
            {
                eprintln!("Error sending event: {}", error);
            };
        });
    }

    fn on_output_level_value_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_output_level_value_changed(move |level| {
            if let Err(error) = synth_sender.send(EventType::UpdateOutputLevel(level).clone()) {
                eprintln!("Error sending event: {}", error);
            };
        });
    }
}
