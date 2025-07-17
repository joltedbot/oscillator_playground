use crate::events::EventType;
use crate::synth::dynamics::Dynamics;
use crate::synth::envelope::{ADSRState, Envelope, GateState};
use crate::synth::lfo::LFO;
use arpeggiator::Arpeggiator;
use cpal::Stream;
use cpal::traits::{DeviceTrait, StreamTrait};
use crossbeam_channel::Receiver;
use device::AudioDevice;
use filter::Filter;
use oscillators::Oscillators;
use std::sync::{Arc, Mutex, MutexGuard};

pub mod arpeggiator;
pub mod device;
pub mod dynamics;
pub mod envelope;
pub mod filter;
pub mod lfo;
pub mod oscillators;

const OUTPUT_LEVEL: f32 = -10.0; // Sets output level to -10.  Change to any dbfs level you want
const UNBLANCED_OUTPUT_LEVEL_ADJUSTMENT: f32 = 3.0;
const LFO_INDEX_FOR_AUTO_PAN: usize = 0;
const LFO_INDEX_FOR_TREMOLO: usize = 1;
const LFO_INDEX_FOR_FILTER_MOD: usize = 2;
const LFO_INDEX_FOR_SUB_OSCILLATOR_MOD: usize = 3;
const LFO_INDEX_FOR_OSCILLATOR1_MOD: usize = 4;
const LFO_INDEX_FOR_OSCILLATOR2_MOD: usize = 5;
const LFO_INDEX_FOR_OSCILLATOR3_MOD: usize = 6;

const OSC_MOD_LFO_INDEX_FOR_SUB: usize = 0;
const OSC_MOD_LFO_INDEX_FOR_OSC1: usize = 1;
const OSC_MOD_LFO_INDEX_FOR_OSC2: usize = 2;
const OSC_MOD_LFO_INDEX_FOR_OSC3: usize = 3;

const LFO_INDEX_FOR_PHASE_DELAY: usize = 7;

const DEFAULT_CENTER_VALUE: f32 = 0.5;
const DEFAULT_AUTO_PAN_CENTER_VALUE: f32 = 1.0;
const DEFAULT_PHASER_CENTER_VALUE: f32 = 87.0;
const DEFAULT_PHASER_WIDTH: f32 = 40.0;
const DEFAULT_LFO_FREQUENCY: f32 = 1.0;
const DEFAULT_COMPRESSOR_RATIO: f32 = 0.5;
const DEFAULT_COMPRESSOR_THRESHOLD: f32 = 0.0;
const DEFAULT_SEQUENCER_NOTE: u32 = 60;
const PHASER_MAX_WIDTH_VALUE: usize = 126;
const ARPEGGIATOR_DEFAULT_RANDOMIZE_STATE: bool = false;
const DEFAULT_BITCRUSHER_DEPTH: u32 = 8;
const DEFAULT_STEREO_WIDTH: f32 = 0.0;

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub enum AmpMode {
    Gate,
    #[default]
    Envelope,
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct LFOParameters {
    is_enabled: bool,
    frequency: f32,
    center_value: f32,
    width: f32,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct DynamicsParameters {
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
pub struct EffectsParameters {
    phaser: LFOParameters,
    bitcrusher_is_enabled: bool,
    bitcrusher_depth: u32,
    stereo_width_is_enabled: bool,
    stereo_width: f32,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct SynthParameters {
    amp_mode: AmpMode,
    output_level: f32,
    output_level_constant: bool,
    auto_pan: LFOParameters,
    tremolo: LFOParameters,
    filter_mod: LFOParameters,
    oscillator_mod_lfos: Vec<LFOParameters>,
    dynamics: DynamicsParameters,
    effects: EffectsParameters,
    arpeggiator: Arpeggiator,
    randomize_arp: bool,
    current_note_frequency: f32,
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
    phaser_buffer: Arc<Mutex<Vec<f32>>>,
}

impl Synth {
    pub fn new(audio_device: AudioDevice) -> Self {
        let sample_rate = audio_device.get_sample_rate();

        // Set up your initial oscillators and set their WaveShape
        let oscillators = Oscillators::new(sample_rate);
        let oscillators_arc = Arc::new(Mutex::new(oscillators));

        // Initialize the modulation module and define your ADSR Envelope
        let envelope = Arc::new(Mutex::new(Envelope::new(sample_rate as u32)));

        let lfo1 = LFO::new(sample_rate);
        let lfo2 = LFO::new(sample_rate);
        let lfo3 = LFO::new(sample_rate);
        let lfo4 = LFO::new(sample_rate);
        let lfo5 = LFO::new(sample_rate);
        let lfo6 = LFO::new(sample_rate);
        let lfo7 = LFO::new(sample_rate);
        let lfo8 = LFO::new(sample_rate);

        let lfos_arc = Arc::new(Mutex::new(vec![
            lfo1, lfo2, lfo3, lfo4, lfo5, lfo6, lfo7, lfo8,
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

        let sub_osc_mod = LFOParameters {
            center_value: DEFAULT_CENTER_VALUE,
            frequency: DEFAULT_LFO_FREQUENCY,
            ..Default::default()
        };

        let osc1_mod = LFOParameters {
            center_value: DEFAULT_CENTER_VALUE,
            frequency: DEFAULT_LFO_FREQUENCY,
            ..Default::default()
        };

        let osc2_mod = LFOParameters {
            center_value: DEFAULT_CENTER_VALUE,
            frequency: DEFAULT_LFO_FREQUENCY,
            ..Default::default()
        };

        let osc3_mod = LFOParameters {
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

        let effects = EffectsParameters {
            phaser,
            bitcrusher_depth: DEFAULT_BITCRUSHER_DEPTH,
            stereo_width: DEFAULT_STEREO_WIDTH,
            ..Default::default()
        };

        let oscillator_mod_lfos = vec![sub_osc_mod, osc1_mod, osc2_mod, osc3_mod];

        let dynamics = DynamicsParameters {
            compressor_ratio: DEFAULT_COMPRESSOR_RATIO,
            compressor_threshold: DEFAULT_COMPRESSOR_THRESHOLD,
            ..Default::default()
        };

        let mut arpeggiator = Arpeggiator::new(vec![DEFAULT_SEQUENCER_NOTE]);
        let randomize_arp = ARPEGGIATOR_DEFAULT_RANDOMIZE_STATE;

        let current_note_frequency = arpeggiator.next_note_frequency(randomize_arp);

        let parameters = SynthParameters {
            amp_mode: AmpMode::Envelope,
            output_level: OUTPUT_LEVEL,
            auto_pan,
            tremolo,
            filter_mod,
            oscillator_mod_lfos,
            output_level_constant: false,
            dynamics,
            effects,
            arpeggiator,
            current_note_frequency,
            randomize_arp,
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

    pub fn run(&mut self, ui_receiver: Receiver<EventType>) {
        self.stream = Some(self.create_audio_engine());

        loop {
            if let Ok(event) = ui_receiver.recv() {
                match event {
                    EventType::UpdateOscillator1Shape(shape) => {
                        let mut oscillators = self.get_oscillators_mutex_lock();
                        let wave_shape = oscillators.get_wave_shape_from_shape_name(shape);
                        oscillators.set_oscillator1_type(wave_shape);
                    }
                    EventType::UpdateOscillator2Shape(shape) => {
                        let mut oscillators = self.get_oscillators_mutex_lock();
                        let wave_shape = oscillators.get_wave_shape_from_shape_name(shape);
                        oscillators.set_oscillator2_type(wave_shape);
                    }
                    EventType::UpdateOscillator3Shape(shape) => {
                        let mut oscillators = self.get_oscillators_mutex_lock();
                        let wave_shape = oscillators.get_wave_shape_from_shape_name(shape);
                        oscillators.set_oscillator3_type(wave_shape);
                    }
                    EventType::UpdateSubOscillatorShape(shape) => {
                        let mut oscillators = self.get_oscillators_mutex_lock();
                        let wave_shape = oscillators.get_wave_shape_from_shape_name(shape);
                        oscillators.set_sub_oscillator_type(wave_shape);
                    }
                    EventType::UpdateOscillator1Level(level) => {
                        let mut oscillators = self.get_oscillators_mutex_lock();
                        oscillators.set_oscillator1_level(level);
                    }
                    EventType::UpdateOscillator2Level(level) => {
                        let mut oscillators = self.get_oscillators_mutex_lock();
                        oscillators.set_oscillator2_level(level);
                    }
                    EventType::UpdateOscillator3Level(level) => {
                        let mut oscillators = self.get_oscillators_mutex_lock();
                        oscillators.set_oscillator3_level(level);
                    }
                    EventType::UpdateSubOscillatorLevel(level) => {
                        let mut oscillators = self.get_oscillators_mutex_lock();
                        oscillators.set_sub_oscillator_level(level);
                    }
                    EventType::UpdateSubOscillatorModFreq(speed) => {
                        let mut parameters = self.get_synth_parameters_mutex_lock();
                        parameters.oscillator_mod_lfos[0].frequency = speed;
                    }
                    EventType::UpdateOscillator1ModFreq(speed) => {
                        let mut parameters = self.get_synth_parameters_mutex_lock();
                        parameters.oscillator_mod_lfos[1].frequency = speed;
                    }
                    EventType::UpdateOscillator2ModFreq(speed) => {
                        let mut parameters = self.get_synth_parameters_mutex_lock();
                        parameters.oscillator_mod_lfos[2].frequency = speed;
                    }
                    EventType::UpdateOscillator3ModFreq(speed) => {
                        let mut parameters = self.get_synth_parameters_mutex_lock();
                        parameters.oscillator_mod_lfos[3].frequency = speed;
                    }
                    EventType::UpdateSubOscillatorModAmount(amount) => {
                        let mut parameters = self.get_synth_parameters_mutex_lock();
                        parameters.oscillator_mod_lfos[0].width = amount;
                    }
                    EventType::UpdateOscillator1ModAmount(amount) => {
                        let mut parameters = self.get_synth_parameters_mutex_lock();
                        parameters.oscillator_mod_lfos[1].width = amount;
                    }
                    EventType::UpdateOscillator2ModAmount(amount) => {
                        let mut parameters = self.get_synth_parameters_mutex_lock();
                        parameters.oscillator_mod_lfos[2].width = amount;
                    }
                    EventType::UpdateOscillator3ModAmount(amount) => {
                        let mut parameters = self.get_synth_parameters_mutex_lock();
                        parameters.oscillator_mod_lfos[3].width = amount;
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
                            (PHASER_MAX_WIDTH_VALUE as f32 - (amount / 2.0)).floor();
                    }
                    EventType::UpdateBitCrusherEnabled(is_enabled) => {
                        let mut parameters = self.get_synth_parameters_mutex_lock();
                        parameters.effects.bitcrusher_is_enabled = is_enabled;
                    }
                    EventType::UpdateBitCrusherAmount(depth) => {
                        let mut parameters = self.get_synth_parameters_mutex_lock();
                        parameters.effects.bitcrusher_depth = depth as u32;
                    }
                    EventType::UpdateStereoWidthEnabled(is_enabled) => {
                        let mut parameters = self.get_synth_parameters_mutex_lock();
                        parameters.effects.stereo_width_is_enabled = is_enabled;
                    }
                    EventType::UpdateStereoWidthAmount(width) => {
                        let mut parameters = self.get_synth_parameters_mutex_lock();
                        parameters.effects.stereo_width = width;
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
                    EventType::ArpeggiatorAddNote(note_number) => {
                        let mut parameters = self.get_synth_parameters_mutex_lock();
                        parameters.arpeggiator.add_note(note_number as u32);
                    }
                    EventType::ArpeggiatorRemoveNote(note_number) => {
                        let mut parameters = self.get_synth_parameters_mutex_lock();
                        parameters.arpeggiator.remove_note(note_number as u32);
                    }
                    EventType::ArpeggiatorRandomEnabled(is_active) => {
                        let mut parameters = self.get_synth_parameters_mutex_lock();
                        parameters.randomize_arp = is_active;
                    }

                    EventType::Start => {
                        self.start();
                    }
                    EventType::Stop => {
                        self.stop();
                    }
                }
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

    fn start(&mut self) {
        if let Some(ref mut stream) = self.stream {
            stream.play().expect("Failed to play audio stream");
        }
    }

    fn stop(&mut self) {
        if let Some(ref mut stream) = self.stream {
            stream.pause().expect("Failed to play audio stream");
        }
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
                    let mut oscillators = oscillators_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    let oscillator1_level = oscillators.get_oscillator1_level();
                    let oscillator2_level = oscillators.get_oscillator2_level();
                    let oscillator3_level = oscillators.get_oscillator3_level();
                    let sub_oscillator_level = oscillators.get_sub_oscillator_level();

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

                    let mut delay_buffer = delay_buffer_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    if delay_buffer.is_empty() {
                        *delay_buffer = vec![0.0; buffer.len()];
                    }

                    for frame in buffer.chunks_mut(number_of_channels) {
                        let sub_oscillator_mod = get_oscillator_mod_value(
                            &mut lfos[LFO_INDEX_FOR_SUB_OSCILLATOR_MOD],
                            OSC_MOD_LFO_INDEX_FOR_SUB,
                            &mut parameters,
                        );

                        let oscillator1_mod = get_oscillator_mod_value(
                            &mut lfos[LFO_INDEX_FOR_OSCILLATOR1_MOD],
                            OSC_MOD_LFO_INDEX_FOR_OSC1,
                            &mut parameters,
                        );

                        let oscillator2_mod = get_oscillator_mod_value(
                            &mut lfos[LFO_INDEX_FOR_OSCILLATOR2_MOD],
                            OSC_MOD_LFO_INDEX_FOR_OSC2,
                            &mut parameters,
                        );

                        let oscillator3_mod = get_oscillator_mod_value(
                            &mut lfos[LFO_INDEX_FOR_OSCILLATOR3_MOD],
                            OSC_MOD_LFO_INDEX_FOR_OSC3,
                            &mut parameters,
                        );

                        let sub_oscillator_sample = oscillators.get_sub_oscillator_next_sample(
                            parameters.current_note_frequency,
                            sub_oscillator_level,
                            sub_oscillator_mod,
                        );

                        let oscillator1_sample = oscillators.get_oscillator1_next_sample(
                            parameters.current_note_frequency,
                            oscillator1_level,
                            oscillator1_mod,
                        );

                        let oscillator2_sample = oscillators.get_oscillator2_next_sample(
                            parameters.current_note_frequency,
                            oscillator2_level,
                            oscillator2_mod,
                        );
                        let oscillator3_sample = oscillators.get_oscillator3_next_sample(
                            parameters.current_note_frequency,
                            oscillator3_level,
                            oscillator3_mod,
                        );

                        let mut oscillator_sum = oscillator1_sample
                            + oscillator2_sample
                            + oscillator3_sample
                            + sub_oscillator_sample;

                        if parameters.effects.phaser.is_enabled {
                            oscillator_sum = get_phased_sample(
                                &mut lfos,
                                &mut parameters,
                                &mut delay_buffer,
                                oscillator_sum,
                            );
                        }

                        let mut balanced_oscillator_level_sum = get_balanced_oscillator_sum(
                            oscillator1_level,
                            oscillator2_level,
                            oscillator3_level,
                            sub_oscillator_level,
                            &mut parameters,
                            oscillator_sum,
                        );

                        if parameters.effects.bitcrusher_is_enabled {
                            balanced_oscillator_level_sum = get_bitcrush_sample(
                                balanced_oscillator_level_sum,
                                parameters.effects.bitcrusher_depth,
                            );
                        }


                        let filter_mod_value = get_filter_mod_value(&mut lfos, &mut parameters);

                        let filtered_sample =
                            filter.filter_sample(balanced_oscillator_level_sum, filter_mod_value);

                        let mut left_sample = filtered_sample;
                        let mut right_sample = filtered_sample;

                        if parameters.effects.stereo_width_is_enabled {
                            (left_sample, right_sample)  = get_stereo_width_samples(
                                left_sample,
                                right_sample,
                            );
                        }

                        if parameters.auto_pan.is_enabled {
                            (left_sample, right_sample) = get_auto_pan_value(
                                &mut lfos,
                                &mut parameters,
                                left_sample,
                                right_sample,
                            );
                        }

                        if parameters.tremolo.is_enabled {
                            (left_sample, right_sample) = get_tremolo_value(
                                &mut lfos,
                                &mut parameters,
                                left_sample,
                                right_sample,
                            );
                        }

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
                                    let randomize = parameters.randomize_arp;
                                    parameters.current_note_frequency =
                                        parameters.arpeggiator.next_note_frequency(randomize);
                                }
                            }
                        } else {
                            match envelope.adsr(parameters.output_level) {
                                ADSRState::Playing(db_adjustment) => {
                                    left_sample *= db_adjustment;
                                    right_sample *= db_adjustment;
                                }
                                ADSRState::Stopped => {
                                    left_sample *= 0.0;
                                    right_sample *= 0.0;
                                    let randomize = parameters.randomize_arp;
                                    parameters.current_note_frequency =
                                        parameters.arpeggiator.next_note_frequency(randomize);
                                }
                            }
                        }

                        if parameters.dynamics.compressor_enabled {
                            (left_sample, right_sample) = get_compressed_samples(
                                &mut parameters,
                                &dynamics,
                                left_sample,
                                right_sample,
                            );
                        }

                        if parameters.dynamics.wavefolder_enabled {
                            (left_sample, right_sample) = get_wavefolded_samples(
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

                        frame[0] = left_sample;
                        frame[1] = right_sample;
                    }
                },
                |err| panic!("an error occurred for the stream: {err}"),
                None,
            )
            .unwrap();

        stream.pause().unwrap();

        stream
    }
}

fn get_filter_mod_value(
    lfos: &mut MutexGuard<Vec<LFO>>,
    parameters: &mut MutexGuard<SynthParameters>,
) -> Option<f32> {
    match parameters.filter_mod.is_enabled {
        true => Some(lfos[LFO_INDEX_FOR_FILTER_MOD].get_next_value(
            parameters.filter_mod.frequency,
            parameters.filter_mod.center_value,
            parameters.filter_mod.width,
        )),
        false => None,
    }
}

fn get_balanced_oscillator_sum(
    oscillator1_level: f32,
    oscillator2_level: f32,
    oscillator3_level: f32,
    sub_oscillator_level: f32,
    parameters: &mut MutexGuard<SynthParameters>,
    oscillator_sum: f32,
) -> f32 {
    match parameters.output_level_constant {
        true => {
            let oscillator_level_sum =
                oscillator1_level + oscillator2_level + oscillator3_level + sub_oscillator_level;

            oscillator_sum / oscillator_level_sum
        }
        false => oscillator_sum / UNBLANCED_OUTPUT_LEVEL_ADJUSTMENT,
    }
}

fn get_oscillator_mod_value(
    lfo: &mut LFO,
    index: usize,
    parameters: &mut MutexGuard<SynthParameters>,
) -> Option<f32> {
    if parameters.oscillator_mod_lfos[index].width > 0.0 {
        Some(lfo.get_next_value(
            parameters.oscillator_mod_lfos[index].frequency,
            parameters.oscillator_mod_lfos[index].center_value,
            parameters.oscillator_mod_lfos[index].width,
        ))
    } else {
        None
    }
}

fn get_tremolo_value(
    lfos: &mut MutexGuard<Vec<LFO>>,
    parameters: &mut MutexGuard<SynthParameters>,
    mut left_sample: f32,
    mut right_sample: f32,
) -> (f32, f32) {
    let tremolo_value = lfos[LFO_INDEX_FOR_TREMOLO].get_next_value(
        parameters.tremolo.frequency,
        parameters.tremolo.center_value,
        parameters.tremolo.width,
    );
    left_sample *= tremolo_value;
    right_sample *= tremolo_value;

    (left_sample, right_sample)
}

fn get_auto_pan_value(
    lfos: &mut MutexGuard<Vec<LFO>>,
    parameters: &mut MutexGuard<SynthParameters>,
    mut left_sample: f32,
    mut right_sample: f32,
) -> (f32, f32) {
    let pan_value = lfos[LFO_INDEX_FOR_AUTO_PAN].get_next_value(
        parameters.auto_pan.frequency,
        parameters.auto_pan.center_value,
        parameters.auto_pan.width,
    );

    left_sample *= pan_value;
    right_sample *= 2.0 - pan_value;

    (left_sample, right_sample)
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

fn get_phased_sample(
    lfos: &mut MutexGuard<Vec<LFO>>,
    parameters: &mut MutexGuard<SynthParameters>,
    delay_buffer: &mut MutexGuard<Vec<f32>>,
    original_sample: f32,
) -> f32 {
    delay_buffer.insert(0, original_sample);

    let _trash = delay_buffer.pop();

    let phase_shift = lfos[LFO_INDEX_FOR_PHASE_DELAY].get_next_value(
        parameters.effects.phaser.frequency,
        parameters.effects.phaser.center_value,
        parameters.effects.phaser.width,
    );
    (original_sample + delay_buffer[PHASER_MAX_WIDTH_VALUE - (phase_shift.round() as usize)]) / 2.0
}

fn get_bitcrush_sample(original_sample: f32, new_bit_depth: u32) -> f32 {

    let bits = (2_u32.pow(new_bit_depth)/2) as f32;

    let quantized = (original_sample.abs() * bits).ceil();

    let mut bitcrushed_sample = quantized / bits;

    if original_sample.is_sign_negative() {
        bitcrushed_sample *= -1.0;
    }

    bitcrushed_sample

}

fn get_stereo_width_samples(left_sample: f32, right_sample: f32) -> (f32, f32) {

    /*

    [code]
// calc coefs
tmp = 1/max(1 + width,2);
coef_M = 1 * tmp;
coef_S = width * tmp;

// then do this per sample
m = (in_left + in_right)*coef_M;
s = (in_right - in_left )*coef_S;

out_left = m - s;
out_right = m + s;
[/code]
     */

    (left_sample, right_sample)
}