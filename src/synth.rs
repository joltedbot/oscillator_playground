use crate::events::EventType;
use crate::synth::dynamics::Dynamics;
use crate::synth::envelope::{ADSRState, Envelope, GateState};
use crate::synth::lfo::LFO;
use crate::synth::oscillators::sine::Sine;
use arpeggiator::{Arpeggiator, ArpeggiatorType, FIRST_REST_NOTE};
use constants::*;
use cpal::Stream;
use cpal::traits::{DeviceTrait, StreamTrait};
use crossbeam_channel::Receiver;
use device::AudioDevice;
use filter::Filter;
use oscillators::{Oscillators, WaveShape};
use std::sync::{Arc, Mutex, MutexGuard};

pub mod arpeggiator;
mod constants;
pub mod device;
pub mod dynamics;
mod effects;
pub mod envelope;
pub mod filter;
pub mod lfo;
pub mod oscillators;

#[derive(Default, Copy, Clone, Debug, PartialEq)]
enum AmpMode {
    Gate,
    #[default]
    Envelope,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub enum MidiState {
    #[default]
    Rest,
    NoteOn,
    NoteHold,
    NoteOff,
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
struct LFOParameters {
    is_enabled: bool,
    frequency: f32,
    center_value: f32,
    width: f32,
}

#[derive(Default, Clone, Debug, PartialEq)]
struct DynamicsParameters {
    compressor_enabled: bool,
    compressor_threshold: f32,
    compressor_ratio: f32,
    wavefolder_enabled: bool,
    wavefolder_threshold: f32,
    wavefolder_ratio: f32,
    limiter_enabled: bool,
    limiter_threshold: f32,
    clipper_enabled: bool,
    clipper_threshold: f32,
}

#[derive(Default, Clone, Debug, PartialEq)]
struct EffectsParameters {
    phaser: LFOParameters,
    bitcrusher_is_enabled: bool,
    bitcrusher_depth: u32,
    wave_shaper_is_enabled: bool,
    wave_shaper: f32,
}

#[derive(Default, Clone, Debug, PartialEq)]
struct SynthParameters {
    amp_mode: AmpMode,
    output_level: f32,
    output_level_constant: bool,
    auto_pan: LFOParameters,
    tremolo: LFOParameters,
    filter_mod: LFOParameters,
    filter_mod_shape: WaveShape,
    oscillator_mod_lfos: Vec<LFOParameters>,
    current_midi_note: u16,
    current_midi_state: MidiState,
    dynamics: DynamicsParameters,
    effects: EffectsParameters,
    arpeggiator: Arpeggiator,
    arpeggiator_type: ArpeggiatorType,
    arpeggiator_is_active: bool,
    audio_output_channel_indexes: (usize, Option<usize>),
}

pub struct Synth {
    stream: Option<Stream>,
    audio_device: AudioDevice,
    oscillators: Arc<Mutex<Oscillators>>,
    envelope: Arc<Mutex<Envelope>>,
    lfos: Arc<Mutex<Vec<LFO>>>,
    filter: Arc<Mutex<Filter>>,
    dynamics: Arc<Mutex<Dynamics>>,
    parameters: Arc<Mutex<SynthParameters>>,
    phaser_buffer: Arc<Mutex<Vec<(f32, f32)>>>,
}

impl Synth {
    pub fn new(audio_device: AudioDevice) -> Self {
        let sample_rate = audio_device.get_sample_rate();

        // Set up your initial oscillators and set their WaveShape
        let oscillators = Oscillators::new(sample_rate);
        let oscillators_arc = Arc::new(Mutex::new(oscillators));

        // Initialize the modulation module and define your ADSR Envelope
        let envelope = Arc::new(Mutex::new(Envelope::new(sample_rate as u32)));

        let lfos_arc = Arc::new(Mutex::new(vec![
            LFO::new(Box::new(Sine::new(sample_rate))),
            LFO::new(Box::new(Sine::new(sample_rate))),
            LFO::new(Box::new(Sine::new(sample_rate))),
            LFO::new(Box::new(Sine::new(sample_rate))),
            LFO::new(Box::new(Sine::new(sample_rate))),
            LFO::new(Box::new(Sine::new(sample_rate))),
            LFO::new(Box::new(Sine::new(sample_rate))),
            LFO::new(Box::new(Sine::new(sample_rate))),
        ]));

        let filter = Filter::new(sample_rate);
        let filter_arc = Arc::new(Mutex::new(filter));

        let dynamic = Dynamics::new();
        let dynamic_arc = Arc::new(Mutex::new(dynamic));

        let auto_pan = LFOParameters {
            center_value: DEFAULT_AUTO_PAN_CENTER_VALUE,
            frequency: DEFAULT_LFO_FREQUENCY,
            ..Default::default()
        };

        let tremolo = LFOParameters {
            center_value: DEFAULT_CENTER_VALUE,
            frequency: DEFAULT_LFO_FREQUENCY,
            ..Default::default()
        };

        let filter_mod = LFOParameters {
            center_value: DEFAULT_CENTER_VALUE,
            frequency: DEFAULT_LFO_FREQUENCY,
            ..Default::default()
        };

        let phaser = LFOParameters {
            center_value: DEFAULT_PHASER_CENTER_VALUE,
            frequency: DEFAULT_LFO_FREQUENCY,
            width: DEFAULT_PHASER_WIDTH,
            ..Default::default()
        };

        let sub_osc_mod = LFOParameters {
            center_value: DEFAULT_OSC_MOD_CENTER_VALUE,
            frequency: DEFAULT_OSC_MOD_FREQUENCY,
            ..Default::default()
        };

        let osc1_mod = LFOParameters {
            center_value: DEFAULT_OSC_MOD_CENTER_VALUE,
            frequency: DEFAULT_OSC_MOD_FREQUENCY,
            ..Default::default()
        };

        let osc2_mod = LFOParameters {
            center_value: DEFAULT_OSC_MOD_CENTER_VALUE,
            frequency: DEFAULT_OSC_MOD_FREQUENCY,
            ..Default::default()
        };

        let osc3_mod = LFOParameters {
            center_value: DEFAULT_OSC_MOD_CENTER_VALUE,
            frequency: DEFAULT_OSC_MOD_FREQUENCY,
            ..Default::default()
        };

        let oscillator_mod_lfos = vec![sub_osc_mod, osc1_mod, osc2_mod, osc3_mod];

        let dynamics = DynamicsParameters {
            compressor_ratio: DEFAULT_COMPRESSOR_RATIO,
            compressor_threshold: DEFAULT_COMPRESSOR_THRESHOLD,
            ..Default::default()
        };

        let effects = EffectsParameters {
            phaser,
            bitcrusher_depth: DEFAULT_BIT_CRUSHER_DEPTH,
            wave_shaper: DEFAULT_WAVE_SHAPER_AMOUNT,
            ..Default::default()
        };

        let current_midi_note = DEFAULT_SEQUENCER_NOTE;
        let arpeggiator = Arpeggiator::new(vec![current_midi_note]);

        let parameters = SynthParameters {
            amp_mode: AmpMode::Envelope,
            output_level: OUTPUT_LEVEL,
            auto_pan,
            tremolo,
            filter_mod,
            filter_mod_shape: Default::default(),
            oscillator_mod_lfos,
            current_midi_note,
            current_midi_state: Default::default(),
            output_level_constant: true,
            dynamics,
            effects,
            arpeggiator,
            arpeggiator_type: Default::default(),
            arpeggiator_is_active: false,
            audio_output_channel_indexes: (
                DEFAULT_AUDIO_OUTPUT_LEFT_FRAME_INDEX,
                Some(DEFAULT_AUDIO_OUTPUT_RIGHT_FRAME_INDEX),
            ),
        };

        Self {
            stream: None,
            audio_device,
            envelope,
            oscillators: oscillators_arc,
            lfos: lfos_arc,
            filter: filter_arc,
            dynamics: dynamic_arc,
            parameters: Arc::new(Mutex::new(parameters)),
            phaser_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn run(&mut self, synth_receiver: Receiver<EventType>) {
        self.stream = Some(self.create_audio_engine());

        while let Ok(event) = synth_receiver.recv() {
            match event {
                EventType::UpdateOscillatorShape(shape, oscillator) => {
                    let mut oscillators = self.get_oscillators_mutex_lock();
                    let wave_shape = oscillators.get_wave_shape_from_shape_name(shape);
                    oscillators.set_oscillator_type(wave_shape, oscillator);
                }
                EventType::UpdateOscillatorTuning(interval, oscillator) => {
                    let mut oscillators = self.get_oscillators_mutex_lock();
                    oscillators.set_oscillator_interval(interval, oscillator);
                }
                EventType::UpdateOscillatorLevel(level, oscillator) => {
                    let mut oscillators = self.get_oscillators_mutex_lock();
                    oscillators.set_oscillator_level(level, oscillator);
                }
                EventType::UpdateOscillatorSpecificParameters(parameters, oscillator) => {
                    let mut oscillators = self.get_oscillators_mutex_lock();
                    oscillators.set_shape_specific_parameters(parameters, oscillator);
                }
                EventType::UpdateOscillatorShaperAmount(amount, oscillator) => {
                    let mut oscillators = self.get_oscillators_mutex_lock();
                    oscillators.set_oscillator_shaper_amount(amount, oscillator);
                }
                EventType::UpdateOscillatorModFreq(speed, oscillator) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.oscillator_mod_lfos[oscillator as usize].frequency = speed;
                }
                EventType::UpdateOscillatorModAmount(amount, oscillator) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.oscillator_mod_lfos[oscillator as usize].width = amount;
                }
                EventType::UpdateOscillatorDetuneActive(is_active, detune_amount) => {
                    let mut oscillators = self.get_oscillators_mutex_lock();

                    if is_active {
                        oscillators.enable_unison(detune_amount);
                    } else {
                        oscillators.disable_unison();
                    }
                }
                EventType::UpdateOscillatorDetuneValue(detune_amount) => {
                    let mut oscillators = self.get_oscillators_mutex_lock();
                    oscillators.enable_unison(detune_amount);
                }
                EventType::UpdateOutputLevel(level) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.output_level = level as f32;
                }
                EventType::UpdateOutputLevelConstant(is_active) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.output_level_constant = is_active;
                }
                EventType::UpdateEnvelopeAttack(milliseconds) => {
                    let mut envelope = self.get_envelope_mutex_lock();
                    envelope.set_attack_milliseconds(milliseconds.unsigned_abs());
                }
                EventType::UpdateEnvelopeDecay(milliseconds) => {
                    let mut envelope = self.get_envelope_mutex_lock();
                    envelope.set_decay_milliseconds(milliseconds.unsigned_abs());
                }
                EventType::UpdateEnvelopeRelease(milliseconds) => {
                    let mut envelope = self.get_envelope_mutex_lock();
                    envelope.set_release_milliseconds(milliseconds.unsigned_abs());
                }
                EventType::UpdateADSRNoteLength(milliseconds) => {
                    let mut envelope = self.get_envelope_mutex_lock();
                    envelope.set_sustain_milliseconds(milliseconds.unsigned_abs());
                }
                EventType::UpdateEnvelopeSustainLevel(level) => {
                    let mut envelope = self.get_envelope_mutex_lock();
                    envelope.set_sustain_level_below_output_level_in_dbfs(level as f32);
                }
                EventType::UpdateAmpModeEnvelopeEnabled(is_enabled) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    if is_enabled {
                        parameters.amp_mode = AmpMode::Envelope;
                    } else {
                        parameters.amp_mode = AmpMode::Gate;
                    }
                }
                EventType::UpdateGateDutyCycle(duty_cycle) => {
                    let mut envelope = self.get_envelope_mutex_lock();
                    envelope.set_gate_duty_cycle(duty_cycle);
                }
                EventType::UpdateGateNoteLength(note_length) => {
                    let mut envelope = self.get_envelope_mutex_lock();
                    envelope.set_gate_note_length(note_length.unsigned_abs());
                }
                EventType::UpdateFilterCutoffValue(cutoff) => {
                    let mut filter = self.get_filter_mutex_lock();
                    filter.set_cutoff_frequency(cutoff as f32);
                }
                EventType::UpdateFilterResonanceValue(level) => {
                    let mut filter = self.get_filter_mutex_lock();
                    filter.set_resonance(level);
                }
                EventType::UpdateFilterNumberOfPoles(number_of_poles) => {
                    let mut filter = self.get_filter_mutex_lock();
                    filter.set_number_of_poles(number_of_poles);
                }
                EventType::ResyncOscillators => {
                    let mut oscillators = self.get_oscillators_mutex_lock();
                    oscillators.reset();
                }
                EventType::ResyncOscillatorLFOs => {
                    let mut lfos = self.get_lfo_mutex_lock();
                    lfos[LFO_INDEX_FOR_SUB_OSCILLATOR_MOD].reset();
                    lfos[LFO_INDEX_FOR_OSCILLATOR1_MOD].reset();
                    lfos[LFO_INDEX_FOR_OSCILLATOR2_MOD].reset();
                    lfos[LFO_INDEX_FOR_OSCILLATOR3_MOD].reset();
                }
                EventType::UpdateAutoPanEnabled(is_enabled) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.auto_pan.is_enabled = is_enabled;
                }
                EventType::UpdateAutoPanSpeed(speed_hz) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.auto_pan.frequency = speed_hz;
                }
                EventType::UpdateAutoPanWidth(width) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.auto_pan.width = width;
                }
                EventType::UpdateTremoloEnabled(is_enabled) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.tremolo.is_enabled = is_enabled;
                }
                EventType::UpdateTremoloSpeed(speed_hz) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.tremolo.frequency = speed_hz;
                }
                EventType::UpdateTremoloDepth(depth) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.tremolo.width = depth;
                    parameters.tremolo.center_value = 1.0 - (depth / 2.0);
                }
                EventType::UpdateFilterModEnabled(is_enabled) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.filter_mod.is_enabled = is_enabled;
                }
                EventType::UpdateFilterModSpeed(speed_hz) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.filter_mod.frequency = speed_hz;
                }
                EventType::UpdateFilterModAmount(amount) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.filter_mod.width = amount;
                    parameters.filter_mod.center_value = 1.0 - (amount / 2.0);
                }
                EventType::UpdateFilterModShape(shape) => {
                    let lfo_arc = self.lfos.clone();
                    let oscillators_arc = self.oscillators.clone();
                    let parameters_arc = self.parameters.clone();

                    let mut lfos = lfo_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    let mut oscillators = oscillators_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    let mut parameters = parameters_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    let filter_mod_shape = oscillators.get_wave_shape_from_shape_name(shape);
                    let filter_mod_lfo =
                        oscillators.get_oscillator_for_wave_shape(&filter_mod_shape);

                    lfos[LFO_INDEX_FOR_FILTER_MOD] = LFO::new(filter_mod_lfo);
                    parameters.filter_mod_shape = filter_mod_shape;
                }
                EventType::UpdatePhaserEnabled(is_enabled) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.effects.phaser.is_enabled = is_enabled;
                }
                EventType::UpdatePhaserSpeed(speed_hz) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.effects.phaser.frequency = speed_hz;
                }
                EventType::UpdatePhaserAmount(amount) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.effects.phaser.width = amount;
                    parameters.effects.phaser.center_value =
                        effects::get_phaser_lfo_center_value_from_amount(amount);
                }
                EventType::UpdateBitCrusherEnabled(is_enabled) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.effects.bitcrusher_is_enabled = is_enabled;
                }
                EventType::UpdateBitCrusherAmount(depth) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.effects.bitcrusher_depth = depth as u32;
                }
                EventType::UpdateWaveShaperEnabled(is_enabled) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.effects.wave_shaper_is_enabled = is_enabled;
                }
                EventType::UpdateWaveShaperAmount(amount) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.effects.wave_shaper = amount;
                }
                EventType::UpdateCompressorActive(is_active) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.dynamics.compressor_enabled = is_active;
                }
                EventType::UpdateCompressorThreshold(threshold) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.dynamics.compressor_threshold = threshold;
                }
                EventType::UpdateCompressorRatio(ratio) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.dynamics.compressor_ratio = ratio;
                }
                EventType::UpdateWavefolderActive(is_active) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.dynamics.wavefolder_enabled = is_active;
                }
                EventType::UpdateWavefolderThreshold(threshold) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.dynamics.wavefolder_threshold = threshold;
                }
                EventType::UpdateWavefolderRatio(ratio) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.dynamics.wavefolder_ratio = ratio;
                }
                EventType::UpdateLimiterActive(is_active) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.dynamics.limiter_enabled = is_active;
                }
                EventType::UpdateLimiterThreshold(threshold) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.dynamics.limiter_threshold = threshold;
                }
                EventType::UpdateClipperActive(is_active) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.dynamics.clipper_enabled = is_active;
                }
                EventType::UpdateClipperThreshold(threshold) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.dynamics.clipper_threshold = threshold;
                }
                EventType::ArpeggiatorActive(is_active) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.arpeggiator_is_active = is_active;
                }
                EventType::ArpeggiatorAddNote(note_number) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.arpeggiator.add_note(note_number as u16);
                }
                EventType::ArpeggiatorRemoveNote(note_number) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    parameters.arpeggiator.remove_note(note_number as u16);
                }
                EventType::ArpeggiatorRandomEnabled(is_active) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    if is_active {
                        parameters.arpeggiator_type = ArpeggiatorType::Randomize;
                    } else {
                        parameters.arpeggiator_type = ArpeggiatorType::NoteOrder;
                    }
                }
                EventType::MidiNoteOn(note_number) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    if !parameters.arpeggiator_is_active {
                        parameters.current_midi_note = note_number as u16;
                        parameters.current_midi_state = MidiState::NoteOn;
                    }
                }
                EventType::MidiNoteOff(note_number) => {
                    let mut parameters = self.get_synth_parameters_mutex_lock();
                    if !parameters.arpeggiator_is_active
                        && parameters.current_midi_note == note_number as u16
                    {
                        parameters.current_midi_state = MidiState::NoteOff;
                    }
                }
                EventType::UpdateAudioDevice(device) => {
                    {
                        let mut parameters = self.get_synth_parameters_mutex_lock();
                        parameters.audio_output_channel_indexes = (
                            DEFAULT_AUDIO_OUTPUT_LEFT_FRAME_INDEX,
                            Some(DEFAULT_AUDIO_OUTPUT_RIGHT_FRAME_INDEX),
                        );
                    }
                    match self.audio_device.update_audio_device(&device) {
                        Err(error) => eprintln!("Error updating audio device: {error}"),
                        Ok(_) => {
                            self.stream = Some(self.create_audio_engine());
                        }
                    }
                }
                EventType::UpdateAudioChannels(left, right) => {
                    {
                        let mut parameters = self.get_synth_parameters_mutex_lock();
                        parameters.audio_output_channel_indexes =
                            get_channel_frame_indexes_from_channel_names(&left, &right);
                    }

                    self.stream = Some(self.create_audio_engine());
                }
                _ => {}
            }
        }
    }

    fn get_oscillators_mutex_lock(&mut self) -> MutexGuard<Oscillators> {
        self.oscillators
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn get_synth_parameters_mutex_lock(&mut self) -> MutexGuard<SynthParameters> {
        self.parameters
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn get_envelope_mutex_lock(&mut self) -> MutexGuard<Envelope> {
        self.envelope
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn get_lfo_mutex_lock(&mut self) -> MutexGuard<Vec<LFO>> {
        self.lfos
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn get_filter_mutex_lock(&mut self) -> MutexGuard<Filter> {
        self.filter
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn create_audio_engine(&mut self) -> Stream {
        let stream_config = self.audio_device.get_stream_config();
        let output_device = self.audio_device.get_output_device();
        let number_of_channels = self.audio_device.get_number_of_channels();

        let envelope_arc = self.envelope.clone();
        let oscillators_arc = self.oscillators.clone();
        let filter_arc = self.filter.clone();
        let lfo_arc = self.lfos.clone();
        let dynamics_arc = self.dynamics.clone();
        let parameters_arc = self.parameters.clone();
        let delay_buffer_arc = self.phaser_buffer.clone();

        let stream = output_device
            .build_output_stream(
                stream_config,
                move |buffer: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut lfos = lfo_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    let mut envelope = envelope_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    let mut parameters = parameters_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    let mut filter = filter_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    let dynamics = dynamics_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    let mut phaser_delay_buffer = delay_buffer_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    if phaser_delay_buffer.is_empty() {
                        *phaser_delay_buffer = vec![(0.0, 0.0); buffer.len()];
                    }

                    let mut oscillators = oscillators_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    let sub_oscillator_frequency = get_frequency_from_midi_note_and_osc_interval(
                        &parameters.arpeggiator,
                        parameters.current_midi_note,
                        oscillators.get_oscillator_interval(0),
                    );

                    let oscillator1_frequency = get_frequency_from_midi_note_and_osc_interval(
                        &parameters.arpeggiator,
                        parameters.current_midi_note,
                        oscillators.get_oscillator_interval(1),
                    );

                    let oscillator2_frequency = get_frequency_from_midi_note_and_osc_interval(
                        &parameters.arpeggiator,
                        parameters.current_midi_note,
                        oscillators.get_oscillator_interval(2),
                    );

                    let oscillator3_frequency = get_frequency_from_midi_note_and_osc_interval(
                        &parameters.arpeggiator,
                        parameters.current_midi_note,
                        oscillators.get_oscillator_interval(3),
                    );

                    let oscillator1_level = oscillators.get_oscillator1_level();
                    let oscillator2_level = oscillators.get_oscillator2_level();
                    let oscillator3_level = oscillators.get_oscillator3_level();
                    let sub_oscillator_level = oscillators.get_sub_oscillator_level();

                    let left_channel_index = parameters.audio_output_channel_indexes.0;
                    let right_channel_index = parameters.audio_output_channel_indexes.1;


                    // Start the processing of individual frames
                    for frame in buffer.chunks_mut(number_of_channels) {
                        let sub_oscillator_modulation = get_oscillator_mod_value(
                            &mut lfos[LFO_INDEX_FOR_SUB_OSCILLATOR_MOD],
                            parameters.oscillator_mod_lfos[OSC_MOD_LFO_INDEX_FOR_SUB],
                        );

                        let oscillator1_modulation = get_oscillator_mod_value(
                            &mut lfos[LFO_INDEX_FOR_OSCILLATOR1_MOD],
                            parameters.oscillator_mod_lfos[OSC_MOD_LFO_INDEX_FOR_OSC1],
                        );

                        let oscillator2_modulation = get_oscillator_mod_value(
                            &mut lfos[LFO_INDEX_FOR_OSCILLATOR2_MOD],
                            parameters.oscillator_mod_lfos[OSC_MOD_LFO_INDEX_FOR_OSC2],
                        );

                        let oscillator3_modulation = get_oscillator_mod_value(
                            &mut lfos[LFO_INDEX_FOR_OSCILLATOR3_MOD],
                            parameters.oscillator_mod_lfos[OSC_MOD_LFO_INDEX_FOR_OSC3],
                        );

                        let sub_oscillator_sample = oscillators.get_sub_oscillator_next_sample(
                            sub_oscillator_frequency,
                            sub_oscillator_level,
                            sub_oscillator_modulation,
                        );

                        let oscillator1_sample = oscillators.get_oscillator1_next_sample(
                            oscillator1_frequency,
                            oscillator1_level,
                            oscillator1_modulation,
                        );

                        let oscillator2_sample = oscillators.get_oscillator2_next_sample(
                            oscillator2_frequency,
                            oscillator2_level,
                            oscillator2_modulation,
                        );

                        let oscillator3_sample = oscillators.get_oscillator3_next_sample(
                            oscillator3_frequency,
                            oscillator3_level,
                            oscillator3_modulation,
                        );

                        let oscillator_sample_sum = oscillator1_sample
                            + oscillator2_sample
                            + oscillator3_sample
                            + sub_oscillator_sample;

                        let oscillator_level_sum = oscillator1_level
                            + oscillator2_level
                            + oscillator3_level
                            + sub_oscillator_level;

                        let balanced_oscillator_level_sum = get_balanced_oscillator_sum(
                            oscillator_level_sum,
                            parameters.output_level_constant,
                            oscillator_sample_sum,
                        );

                        let filter_mod_value = get_filter_mod_value(
                            &mut lfos[LFO_INDEX_FOR_FILTER_MOD],
                            &mut parameters,
                        );
                        let filtered_sample =
                            filter.filter_sample(balanced_oscillator_level_sum, filter_mod_value);

                        let mut left_sample = filtered_sample;
                        let mut right_sample = filtered_sample;

                        if parameters.auto_pan.is_enabled {
                            (left_sample, right_sample) = effects::get_auto_pan_value(
                                &mut lfos[LFO_INDEX_FOR_AUTO_PAN],
                                &mut parameters.auto_pan,
                                left_sample,
                                right_sample,
                            );
                        }

                        if parameters.effects.phaser.is_enabled {
                            (left_sample, right_sample) = effects::get_phased_sample(
                                &mut lfos[LFO_INDEX_FOR_PHASE_DELAY],
                                &mut parameters.effects.phaser,
                                &mut phaser_delay_buffer,
                                left_sample,
                                right_sample,
                            );
                        }

                        if parameters.tremolo.is_enabled {
                            (left_sample, right_sample) = effects::get_tremolo_value(
                                &mut lfos[LFO_INDEX_FOR_TREMOLO],
                                &mut parameters.tremolo,
                                left_sample,
                                right_sample,
                            );
                        }

                        if parameters.effects.bitcrusher_is_enabled {
                            (left_sample, right_sample) = effects::get_bitcrush_sample(
                                parameters.effects.bitcrusher_depth,
                                left_sample,
                                right_sample,
                            );
                        }

                        if parameters.effects.wave_shaper_is_enabled {
                            (left_sample, right_sample) = effects::get_wave_shaped_sample(
                                parameters.effects.wave_shaper,
                                left_sample,
                                right_sample,
                            );
                        }
                        let arp_is_active = parameters.arpeggiator_is_active;

                        if parameters.amp_mode == AmpMode::Gate {
                            match envelope.gate(parameters.output_level) {
                                GateState::On(db_adjustment) => {
                                    left_sample *= db_adjustment;
                                    right_sample *= db_adjustment;
                                }
                                GateState::Off(db_adjustment) => {
                                    left_sample *= db_adjustment;
                                    right_sample *= db_adjustment;
                                }
                                GateState::End(db_adjustment) => {
                                    left_sample *= db_adjustment;
                                    right_sample *= db_adjustment;
                                    if arp_is_active {
                                        let arpeggiator_type = parameters.arpeggiator_type.clone();
                                        parameters.current_midi_note =
                                            parameters.arpeggiator.next_midi_note(arpeggiator_type);
                                        parameters.current_midi_state = MidiState::NoteOn;
                                    }
                                }
                            }
                        } else {
                            match envelope.adsr(
                                parameters.output_level,
                                &mut parameters.current_midi_state,
                                arp_is_active,
                            ) {
                                ADSRState::Playing(db_adjustment) => {
                                    left_sample *= db_adjustment;
                                    right_sample *= db_adjustment;
                                }
                                ADSRState::Stopped => {
                                    left_sample *= 0.0;
                                    right_sample *= 0.0;

                                    if arp_is_active {
                                        let arpeggiator_type = parameters.arpeggiator_type.clone();
                                        parameters.current_midi_note =
                                            parameters.arpeggiator.next_midi_note(arpeggiator_type);
                                        parameters.current_midi_state = MidiState::NoteOn;
                                    } else {
                                        parameters.current_midi_state = MidiState::Rest;
                                    }
                                }
                            }
                        }

                        if parameters.dynamics.wavefolder_enabled {
                            (left_sample, right_sample) = get_wavefolded_samples(
                                &mut parameters,
                                &dynamics,
                                left_sample,
                                right_sample,
                            );
                        }

                        if parameters.dynamics.compressor_enabled {
                            (left_sample, right_sample) = get_compressed_samples(
                                &mut parameters,
                                &dynamics,
                                left_sample,
                                right_sample,
                            );
                        }

                        if parameters.dynamics.limiter_enabled {
                            (left_sample, right_sample) = get_limited_samples(
                                &mut parameters,
                                &dynamics,
                                left_sample,
                                right_sample,
                            );
                        }

                        if parameters.dynamics.clipper_enabled {
                            (left_sample, right_sample) = get_clipped_samples(
                                &mut parameters,
                                &dynamics,
                                left_sample,
                                right_sample,
                            );
                        }

                        frame[left_channel_index] = left_sample;

                        if number_of_channels > 1 && right_channel_index.is_some() {
                            let right_index = right_channel_index.unwrap();
                            frame[right_index] = right_sample;
                        }
                    }
                },
                |err| panic!("an error occurred for the stream: {err}"),
                None,
            )
            .unwrap();

        stream.play().expect("Failed to play audio stream");

        stream
    }
}

fn get_channel_frame_indexes_from_channel_names(left: &str, right: &str) -> (usize, Option<usize>) {
    let left_channel: usize = left
        .parse()
        .unwrap_or(DEFAULT_AUDIO_OUTPUT_LEFT_FRAME_INDEX);
    let right_channel = right.parse::<usize>().ok();
    let left_channel_index = left_channel - CHANNEL_TO_FRAME_INDEX_OFFSET;
    let right_channel_index = right_channel.map(|channel| channel - CHANNEL_TO_FRAME_INDEX_OFFSET);
    (left_channel_index, right_channel_index)
}

fn get_filter_mod_value(
    lfo: &mut LFO,
    parameters: &mut MutexGuard<SynthParameters>,
) -> Option<f32> {
    match parameters.filter_mod.is_enabled {
        true => Some(lfo.get_next_value(
            parameters.filter_mod.frequency,
            parameters.filter_mod.center_value,
            parameters.filter_mod.width,
        )),
        false => None,
    }
}

fn get_balanced_oscillator_sum(
    oscillator_level_sum: f32,
    output_level_is_constant: bool,
    oscillator_sum: f32,
) -> f32 {
    match output_level_is_constant {
        true => oscillator_sum / oscillator_level_sum,
        false => oscillator_sum / UNBALANCED_OUTPUT_LEVEL_ADJUSTMENT,
    }
}

fn get_oscillator_mod_value(lfo: &mut LFO, lfo_parameters: LFOParameters) -> Option<f32> {
    if lfo_parameters.width > 0.0 {
        Some(lfo.get_next_value(
            lfo_parameters.frequency,
            lfo_parameters.center_value,
            lfo_parameters.width,
        ))
    } else {
        None
    }
}

fn get_compressed_samples(
    parameters: &mut MutexGuard<SynthParameters>,
    dynamics: &MutexGuard<Dynamics>,
    left_sample: f32,
    right_sample: f32,
) -> (f32, f32) {
    let left_sample = dynamics.compress(
        parameters.output_level,
        parameters.dynamics.compressor_threshold,
        parameters.dynamics.compressor_ratio,
        left_sample,
    );

    let right_sample = dynamics.compress(
        parameters.output_level,
        parameters.dynamics.compressor_threshold,
        parameters.dynamics.compressor_ratio,
        right_sample,
    );

    (left_sample, right_sample)
}

fn get_wavefolded_samples(
    parameters: &mut MutexGuard<SynthParameters>,
    dynamics: &MutexGuard<Dynamics>,
    left_sample: f32,
    right_sample: f32,
) -> (f32, f32) {
    let left_sample = dynamics.wavefold(
        parameters.output_level,
        parameters.dynamics.wavefolder_threshold,
        parameters.dynamics.wavefolder_ratio,
        left_sample,
    );
    let right_sample = dynamics.wavefold(
        parameters.output_level,
        parameters.dynamics.wavefolder_threshold,
        parameters.dynamics.wavefolder_ratio,
        right_sample,
    );

    (left_sample, right_sample)
}

fn get_limited_samples(
    parameters: &mut MutexGuard<SynthParameters>,
    dynamics: &MutexGuard<Dynamics>,
    left_sample: f32,
    right_sample: f32,
) -> (f32, f32) {
    let left_sample = dynamics.limit(
        parameters.output_level,
        parameters.dynamics.limiter_threshold,
        left_sample,
    );
    let right_sample = dynamics.limit(
        parameters.output_level,
        parameters.dynamics.limiter_threshold,
        right_sample,
    );

    (left_sample, right_sample)
}

fn get_clipped_samples(
    parameters: &mut MutexGuard<SynthParameters>,
    dynamics: &MutexGuard<Dynamics>,
    left_sample: f32,
    right_sample: f32,
) -> (f32, f32) {
    let left_sample = dynamics.clip(
        parameters.output_level,
        parameters.dynamics.clipper_threshold,
        left_sample,
    );
    let right_sample = dynamics.clip(
        parameters.output_level,
        parameters.dynamics.clipper_threshold,
        right_sample,
    );

    (left_sample, right_sample)
}

fn get_frequency_from_midi_note_and_osc_interval(
    arpeggiator: &Arpeggiator,
    midi_note: u16,
    interval: i32,
) -> f32 {
    if midi_note >= FIRST_REST_NOTE {
        return 0.0;
    }

    if interval.is_positive() {
        return arpeggiator.get_frequency_from_midi_note(midi_note + interval as u16);
    }

    let new_midi_note = (midi_note as i16).saturating_add(interval as i16) as u16;

    arpeggiator.get_frequency_from_midi_note(new_midi_note)
}
