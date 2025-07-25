use super::AppWindow;
use crate::events::EventType;
use crossbeam_channel::Sender;
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

    pub fn create_ui_callbacks(&mut self) {
        self.on_arp_button_pressed();
        self.on_wave_shape_selected();
        self.on_wave_level_selected();
        self.on_wave_fm_amount_selected();
        self.on_wave_pulse_width_selected();
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

    fn on_wave_fm_amount_selected(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_fm_amount_selected(move |fm_amount, oscillator| {
            if let Err(error) =
                synth_sender.send(EventType::UpdateOscillatorFMAmount(fm_amount, oscillator))
            {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_wave_pulse_width_selected(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_pulse_width_selected(move |pulse_width, oscillator| {
            if let Err(error) = synth_sender.send(EventType::UpdateOscillatorPulseWidth(
                pulse_width,
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

        ui.on_wave_mod_speed_changed(move |level, oscillator| {
            if let Err(error) =
                synth_sender.send(EventType::UpdateOscillatorModFreq(level, oscillator))
            {
                eprintln!("Error sending event: {error}",);
            }
        });
    }

    fn on_wave_mod_amount_changed(&mut self) {
        let ui = self.get_ui_reference_from_ui_weak();
        let synth_sender = self.synth_sender.clone();

        ui.on_wave_mod_amount_changed(move |level, oscillator| {
            if let Err(error) =
                synth_sender.send(EventType::UpdateOscillatorModAmount(level, oscillator))
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
}
