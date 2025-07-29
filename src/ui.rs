use super::ApplicationWindow;
use crate::device_manager::DeviceList;
use crate::events::EventType;
use crossbeam_channel::{Receiver, Sender};
use slint::{ModelRc, SharedString, VecModel, Weak};
use std::error::Error;
use std::process::exit;

const DEFAULT_AUDIO_OUTPUT_DEVICE_INDEX: usize = 0;
const DEFAULT_AUDIO_OUTPUT_LEFT_CHANNEL: &str = "1";
const DEFAULT_AUDIO_OUTPUT_RIGHT_CHANNEL: &str = "2";

pub struct UI {
    pub ui: Weak<ApplicationWindow>,
    synth_sender: Sender<EventType>,
    midi_sender: Sender<EventType>,
    ui_sender: Sender<EventType>,
    current_audio_output_device: String,
    audio_output_devices: DeviceList,
}

impl UI {
    pub fn new(
        ui: Weak<ApplicationWindow>,
        synth_sender: Sender<EventType>,
        midi_sender: Sender<EventType>,
        ui_sender: Sender<EventType>,
    ) -> Result<Self, Box<dyn Error>> {
        let ui = Self {
            ui,
            synth_sender,
            midi_sender,
            ui_sender,
            current_audio_output_device: String::new(),
            audio_output_devices: Default::default(),
        };

        Ok(ui)
    }

    pub fn run(&mut self, ui_receiver: Receiver<EventType>) {
        loop {
            if let Ok(event) = ui_receiver.recv() {
                match event {
                    EventType::UpdateMidiPortList(midi_port_list) => {
                        let ui_weak = self.ui.clone();
                        let _ = ui_weak.upgrade_in_event_loop(move |ui| {
                            let midi_input_port_model =
                                get_model_from_string_slice(&midi_port_list);
                            ui.set_midi_input_ports(midi_input_port_model);
                        });
                    }
                    EventType::UpdateOutputDeviceList(audio_device_list) => {
                        let ui_weak = self.ui.clone();

                        let device_was_removed = !self
                            .audio_output_devices
                            .devices
                            .contains(&self.current_audio_output_device);

                        self.audio_output_devices = audio_device_list.clone();

                        if self.current_audio_output_device.is_empty() {
                            self.current_audio_output_device = self.audio_output_devices.devices
                                [DEFAULT_AUDIO_OUTPUT_DEVICE_INDEX]
                                .clone();
                        }

                        let _ = ui_weak.upgrade_in_event_loop(move |ui| {
                            let audio_output_device_model =
                                get_model_from_string_slice(&audio_device_list.devices);
                            ui.set_audio_output_device_list(audio_output_device_model);

                            if device_was_removed {
                                let audio_output_device_model = get_model_from_string_slice(
                                    &audio_device_list.channels[DEFAULT_AUDIO_OUTPUT_DEVICE_INDEX],
                                );
                                ui.set_audio_output_channels(audio_output_device_model);

                                ui.set_audio_output_device(SharedString::from(
                                    audio_device_list.devices[DEFAULT_AUDIO_OUTPUT_DEVICE_INDEX]
                                        .clone(),
                                ));
                                ui.set_audio_output_left_channel(SharedString::from(
                                    DEFAULT_AUDIO_OUTPUT_LEFT_CHANNEL,
                                ));

                                if audio_device_list.channels[DEFAULT_AUDIO_OUTPUT_DEVICE_INDEX]
                                    .len()
                                    > 1
                                {
                                    ui.set_audio_output_right_channel(SharedString::from(
                                        DEFAULT_AUDIO_OUTPUT_RIGHT_CHANNEL,
                                    ));
                                } else {
                                    ui.set_audio_output_right_channel(SharedString::new());
                                }
                            }
                        });
                    }
                    EventType::UpdateAudioDevice(audio_device_name) => {
                        let synth_sender = self.synth_sender.clone();

                        if let Some(device_index) = self
                            .audio_output_devices
                            .devices
                            .iter()
                            .position(|device| &audio_device_name.to_string() == device)
                        {
                            let left_channel = String::from(
                                self.audio_output_devices.channels[device_index][0].clone(),
                            );
                            let right_channel = if self.audio_output_devices.channels[device_index].len() > 1 {
                                String::from(
                                    self.audio_output_devices.channels[device_index][1].clone(),
                                )
                            } else {
                                String::new()
                            };

                            if let Err(error) = synth_sender
                                .send(EventType::UpdateAudioDevice(audio_device_name.to_string()))
                            {
                                eprintln!("Error sending event: {error}",);
                            }

                            if let Err(error) = synth_sender.send(EventType::UpdateAudioChannels(
                                left_channel.clone(),
                                right_channel.clone(),
                            )) {
                                eprintln!("Error sending event: {error}",);
                            }

                            let device_channels =
                                self.audio_output_devices.channels[device_index].clone();

                            let ui_weak = self.ui.clone();
                            let _ = ui_weak.upgrade_in_event_loop(move |ui| {
                                ui.set_audio_output_channels(get_model_from_string_slice(
                                    &device_channels,
                                ));
                                ui.set_audio_output_left_channel(SharedString::from(left_channel));
                                ui.set_audio_output_right_channel(SharedString::from(
                                    right_channel,
                                ));
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn get_ui_reference_from_ui_weak(&mut self) -> ApplicationWindow {
        let ui_weak = self.ui.clone();

        match ui_weak.upgrade() {
            Some(ui) => ui,
            None => {
                eprintln!("Could not upgrade ui mutex");
                exit(1);
            }
        }
    }

    pub fn create_ui_callbacks(&mut self) {
        self.on_wave_shape_selected();
        self.on_wave_level_selected();
        self.on_wave_specific_parameters_selected();
        self.on_wave_tuning_changed();
        self.on_wave_shaper_amount_changed();
        self.on_wave_mod_speed_changed();
        self.on_wave_mod_amount_changed();
        self.on_wave_detune_value_changed();
        self.on_wave_detune_state_changed();
        self.on_output_level_value_changed();
        self.on_output_level_constant_activated();
        self.on_envelope_attack_updated();
        self.on_envelope_decay_updated();
        self.on_envelope_release_updated();
        self.on_adsr_note_length_updated();
        self.on_envelope_sustain_updated();
        self.on_filter_cutoff_value_changed();
        self.on_filter_resonance_value_changed();
        self.on_number_of_poles_selected();
        self.on_resync_oscillators();
        self.on_resync_oscillator_lfos();
        self.on_gate_length_changed();
        self.on_gate_duty_cycle_changed();
        self.on_enable_amp_envelope();
        self.on_auto_pan_activated();
        self.on_auto_pan_speed_changed();
        self.on_auto_pan_width_changed();
        self.on_tremolo_activated();
        self.on_tremolo_speed_changed();
        self.on_tremolo_depth_changed();
        self.on_filter_mod_activated();
        self.on_filter_mod_speed_changed();
        self.on_filter_mod_depth_changed();
        self.on_filter_mod_shape_selected();
        self.on_phaser_activated();
        self.on_phaser_speed_changed();
        self.on_phaser_depth_changed();
        self.on_bitcrusher_activated();
        self.on_bitcrusher_amount_changed();
        self.on_global_wave_shaper_activated();
        self.on_global_wave_shaper_amount_changed();
        self.on_compressor_activated();
        self.on_compressor_threshold_changed();
        self.on_compressor_ratio_changed();
        self.on_wavefolder_activated();
        self.on_wavefolder_threshold_changed();
        self.on_wavefolder_ratio_changed();
        self.on_limiter_activated();
        self.on_limiter_threshold_changed();
        self.on_clipper_activated();
        self.on_clipper_threshold_changed();
        self.on_note_activated();
        self.on_note_deactivated();
        self.on_arpeggiator_random_activated();
        self.on_arp_button_pressed();
        self.on_midi_port_selected();
        self.on_midi_channel_selected();
        self.on_audio_device_selected();
        self.on_audio_channels_selected();
    }

    fn on_wave_shape_selected(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_shape_selected(move |shape, oscillator| {
            if let Err(error) =
                synth_sender.send(EventType::UpdateOscillatorShape(shape, oscillator))
            {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_wave_tuning_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_tuning_changed(move |interval, oscillator| {
            if let Err(error) =
                synth_sender.send(EventType::UpdateOscillatorTuning(interval, oscillator))
            {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_wave_level_selected(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_level_selected(move |level, oscillator| {
            if let Err(error) =
                synth_sender.send(EventType::UpdateOscillatorLevel(level, oscillator))
            {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_wave_specific_parameters_selected(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_specific_parameters_selected(move |parameter1, parameter2, oscillator| {
            if let Err(error) = synth_sender.send(EventType::UpdateOscillatorSpecificParameters(
                (parameter1, parameter2),
                oscillator,
            )) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_wave_shaper_amount_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_shaper_amount_changed(move |level, oscillator| {
            if let Err(error) =
                synth_sender.send(EventType::UpdateOscillatorShaperAmount(level, oscillator))
            {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_wave_mod_speed_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_mod_speed_changed(move |speed_hz, oscillator| {
            if let Err(error) =
                synth_sender.send(EventType::UpdateOscillatorModFreq(speed_hz, oscillator))
            {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_wave_mod_amount_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_mod_amount_changed(move |amount, oscillator| {
            if let Err(error) =
                synth_sender.send(EventType::UpdateOscillatorModAmount(amount, oscillator))
            {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_arp_button_pressed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_arp_button_pressed(move |is_active| {
            if let Err(error) = synth_sender.send(EventType::ArpeggiatorActive(is_active)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_wave_detune_value_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_detune_value_changed(move |detune_amount| {
            if let Err(error) =
                synth_sender.send(EventType::UpdateOscillatorDetuneValue(detune_amount))
            {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_wave_detune_state_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_detune_state_changed(move |is_active, detune_amount| {
            if let Err(error) = synth_sender.send(EventType::UpdateOscillatorDetuneActive(
                is_active,
                detune_amount,
            )) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_output_level_value_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_output_level_value_changed(move |level| {
            if let Err(error) = synth_sender.send(EventType::UpdateOutputLevel(level)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_output_level_constant_activated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_output_level_constant_activated(move |is_active| {
            if let Err(error) = synth_sender.send(EventType::UpdateOutputLevelConstant(is_active)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_envelope_attack_updated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_envelope_attack_updated(move |milliseconds| {
            if let Err(error) = synth_sender.send(EventType::UpdateEnvelopeAttack(milliseconds)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_envelope_decay_updated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_envelope_decay_updated(move |milliseconds| {
            if let Err(error) = synth_sender.send(EventType::UpdateEnvelopeDecay(milliseconds)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_envelope_release_updated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_envelope_release_updated(move |milliseconds| {
            if let Err(error) = synth_sender.send(EventType::UpdateEnvelopeRelease(milliseconds)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_envelope_sustain_updated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_envelope_sustain_updated(move |level| {
            if let Err(error) = synth_sender.send(EventType::UpdateEnvelopeSustainLevel(level)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_adsr_note_length_updated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_adsr_note_length_updated(move |milliseconds| {
            if let Err(error) = synth_sender.send(EventType::UpdateADSRNoteLength(milliseconds)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_filter_cutoff_value_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_filter_cutoff_value_changed(move |cutoff| {
            if let Err(error) = synth_sender.send(EventType::UpdateFilterCutoffValue(cutoff)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_filter_resonance_value_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_filter_resonance_value_changed(move |level| {
            if let Err(error) = synth_sender.send(EventType::UpdateFilterResonanceValue(level)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_number_of_poles_selected(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_number_of_poles_selected(move |number_of_poles| {
            if let Err(error) =
                synth_sender.send(EventType::UpdateFilterNumberOfPoles(number_of_poles))
            {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_resync_oscillators(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_resync_oscillators(move || {
            if let Err(error) = synth_sender.send(EventType::ResyncOscillators.clone()) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_resync_oscillator_lfos(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_resync_oscillator_lfos(move || {
            if let Err(error) = synth_sender.send(EventType::ResyncOscillatorLFOs.clone()) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_enable_amp_envelope(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_enable_amp_envelope(move |is_enabled| {
            if let Err(error) =
                synth_sender.send(EventType::UpdateAmpModeEnvelopeEnabled(is_enabled))
            {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_gate_duty_cycle_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_gate_duty_cycle_changed(move |duty_cycle| {
            if let Err(error) = synth_sender.send(EventType::UpdateGateDutyCycle(duty_cycle)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_gate_length_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_gate_length_changed(move |milliseconds| {
            if let Err(error) = synth_sender.send(EventType::UpdateGateNoteLength(milliseconds)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_auto_pan_activated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_auto_pan_activated(move |is_active| {
            if let Err(error) = synth_sender.send(EventType::UpdateAutoPanEnabled(is_active)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_auto_pan_speed_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_auto_pan_speed_changed(move |speed_hz| {
            if let Err(error) = synth_sender.send(EventType::UpdateAutoPanSpeed(speed_hz)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_auto_pan_width_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_auto_pan_width_changed(move |width| {
            if let Err(error) = synth_sender.send(EventType::UpdateAutoPanWidth(width)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_tremolo_activated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_tremolo_activated(move |is_active| {
            if let Err(error) = synth_sender.send(EventType::UpdateTremoloEnabled(is_active)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_tremolo_speed_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_tremolo_speed_changed(move |speed_hz| {
            if let Err(error) = synth_sender.send(EventType::UpdateTremoloSpeed(speed_hz)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_tremolo_depth_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_tremolo_depth_changed(move |depth| {
            if let Err(error) = synth_sender.send(EventType::UpdateTremoloDepth(depth)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_filter_mod_activated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_filter_mod_activated(move |is_active| {
            if let Err(error) = synth_sender.send(EventType::UpdateFilterModEnabled(is_active)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_filter_mod_speed_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_filter_mod_speed_changed(move |speed_hz| {
            if let Err(error) = synth_sender.send(EventType::UpdateFilterModSpeed(speed_hz)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_filter_mod_depth_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_filter_mod_amount_changed(move |amount| {
            if let Err(error) = synth_sender.send(EventType::UpdateFilterModAmount(amount)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_filter_mod_shape_selected(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_filter_mod_shape_selected(move |shape| {
            if let Err(error) = synth_sender.send(EventType::UpdateFilterModShape(shape)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_phaser_activated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_phaser_activated(move |is_active| {
            if let Err(error) = synth_sender.send(EventType::UpdatePhaserEnabled(is_active)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_phaser_speed_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_phaser_speed_changed(move |speed_hz| {
            if let Err(error) = synth_sender.send(EventType::UpdatePhaserSpeed(speed_hz)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_phaser_depth_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_phaser_amount_changed(move |amount| {
            if let Err(error) = synth_sender.send(EventType::UpdatePhaserAmount(amount)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_bitcrusher_activated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_bitcrusher_activated(move |is_active| {
            if let Err(error) = synth_sender.send(EventType::UpdateBitCrusherEnabled(is_active)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_bitcrusher_amount_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_bitcrusher_amount_changed(move |amount| {
            if let Err(error) = synth_sender.send(EventType::UpdateBitCrusherAmount(amount)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_global_wave_shaper_activated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_global_wave_shaper_activated(move |is_active| {
            if let Err(error) = synth_sender.send(EventType::UpdateWaveShaperEnabled(is_active)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_global_wave_shaper_amount_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_global_wave_shaper_amount_changed(move |amount| {
            if let Err(error) = synth_sender.send(EventType::UpdateWaveShaperAmount(amount)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_compressor_activated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_compressor_activated(move |is_active| {
            if let Err(error) = synth_sender.send(EventType::UpdateCompressorActive(is_active)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_compressor_threshold_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_compressor_threshold_changed(move |threshold| {
            if let Err(error) = synth_sender.send(EventType::UpdateCompressorThreshold(threshold)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_compressor_ratio_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_compressor_ratio_changed(move |ratio| {
            if let Err(error) = synth_sender.send(EventType::UpdateCompressorRatio(ratio)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_wavefolder_activated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wavefolder_activated(move |is_active| {
            if let Err(error) = synth_sender.send(EventType::UpdateWavefolderActive(is_active)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_wavefolder_threshold_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wavefolder_threshold_changed(move |threshold| {
            if let Err(error) = synth_sender.send(EventType::UpdateWavefolderThreshold(threshold)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_wavefolder_ratio_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wavefolder_ratio_changed(move |ratio| {
            if let Err(error) = synth_sender.send(EventType::UpdateWavefolderRatio(ratio)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_limiter_activated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_limiter_activated(move |is_active| {
            if let Err(error) = synth_sender.send(EventType::UpdateLimiterActive(is_active)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_limiter_threshold_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_limiter_threshold_changed(move |threshold| {
            if let Err(error) = synth_sender.send(EventType::UpdateLimiterThreshold(threshold)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_clipper_activated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_clipper_activated(move |is_active| {
            if let Err(error) = synth_sender.send(EventType::UpdateClipperActive(is_active)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_clipper_threshold_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_clipper_threshold_changed(move |threshold| {
            if let Err(error) = synth_sender.send(EventType::UpdateClipperThreshold(threshold)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_note_activated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_note_activated(move |note_number| {
            if let Err(error) = synth_sender.send(EventType::ArpeggiatorAddNote(note_number)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_note_deactivated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_note_deactivated(move |note_number| {
            if let Err(error) = synth_sender.send(EventType::ArpeggiatorRemoveNote(note_number)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_arpeggiator_random_activated(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_arpeggiator_random_activated(move |is_active| {
            if let Err(error) = synth_sender.send(EventType::ArpeggiatorRandomEnabled(is_active)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_midi_port_selected(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let midi_sender = self.midi_sender.clone();

        ui.on_midi_port_selected(move |port_index| {
            if let Err(error) = midi_sender.send(EventType::UpdateMidiPort(port_index)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_midi_channel_selected(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let midi_sender = self.midi_sender.clone();

        ui.on_midi_channel_selected(move |channel| {
            if let Err(error) = midi_sender.send(EventType::UpdateMidiChannel(channel)) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_audio_device_selected(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let ui_sender = self.ui_sender.clone();

        ui.on_audio_device_selected(move |device_name| {
            if let Err(error) = ui_sender.send(EventType::UpdateAudioDevice(device_name.to_string())) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_audio_channels_selected(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_audio_channels_selected(move |left, right| {
            if let Err(error) = synth_sender.send(EventType::UpdateAudioChannels(
                left.to_string(),
                right.to_string(),
            )) {
                eprintln!("Error sending event: {error}",);
            }
        });
    }
}

fn get_model_from_string_slice(devices: &Vec<String>) -> ModelRc<SharedString> {
    let name_list: Vec<SharedString> = devices.iter().map(SharedString::from).collect();
    ModelRc::new(VecModel::from_slice(name_list.as_slice()))
}
