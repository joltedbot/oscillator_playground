use crate::events::EventType;
use crate::synth::envelope::GateState;
#[allow(dead_code)]
use crate::synth::envelope::{ADSRState, Envelope};
use crate::synth::lfo::LFO;
use cpal::Stream;
use cpal::traits::{DeviceTrait, StreamTrait};
use crossbeam_channel::Receiver;
use device::AudioDevice;
use filter::Filter;
use oscillators::Oscillators;
use sequencer::Sequencer;
use std::sync::{Arc, Mutex, MutexGuard};

pub mod device;
pub mod dynamics;
pub mod envelope;
pub mod filter;
pub mod lfo;
pub mod oscillators;
pub mod sequencer;

const OUTPUT_LEVEL: f32 = -10.0; // Sets output level to -10.  Change to any dbfs level you want
const LFO_INDEX_FOR_AUTO_PAN: usize = 0;
const LFO_INDEX_FOR_TREMOLO: usize = 1;
const LFO_INDEX_FOR_FILTER_MOD: usize = 2;
const DEFAULT_CENTER_FREQUENCY: f32 = 0.5;
const DEFAULT_LFO_FREQUENCY: f32 = 1.0;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AmpMode {
    Gate,
    Envelope,
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct LFOInstance {
    is_enabled: bool,
    frequency: f32,
    center_frequency: f32,
    width: f32,
}

pub struct Synth {
    stream: Option<Stream>,
    audio_device: AudioDevice,
    amp_mode: Arc<Mutex<AmpMode>>,
    oscillators: Arc<Mutex<Oscillators>>,
    envelope: Arc<Mutex<Envelope>>,
    lfos: Arc<Mutex<Vec<LFO>>>,
    filter: Arc<Mutex<Filter>>,
    output_level: Arc<Mutex<f32>>,
    auto_pan: Arc<Mutex<LFOInstance>>,
    tremolo: Arc<Mutex<LFOInstance>>,
    filter_mod: Arc<Mutex<LFOInstance>>,
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

        let lfos_arc = Arc::new(Mutex::new(vec![lfo1, lfo2, lfo3]));

        let filter = Filter::new(sample_rate);
        let filter_arc = Arc::new(Mutex::new(filter));

        let auto_pan = Arc::new(Mutex::new(LFOInstance {
            center_frequency: DEFAULT_CENTER_FREQUENCY,
            frequency: DEFAULT_LFO_FREQUENCY,
            ..Default::default()
        }));

        let tremolo = Arc::new(Mutex::new(LFOInstance {
            center_frequency: DEFAULT_CENTER_FREQUENCY,
            frequency: DEFAULT_LFO_FREQUENCY,
            ..Default::default()
        }));

        let filter_mod = Arc::new(Mutex::new(LFOInstance {
            center_frequency: DEFAULT_CENTER_FREQUENCY,
            frequency: DEFAULT_LFO_FREQUENCY,
            ..Default::default()
        }));

        Self {
            stream: None,
            amp_mode: Arc::new(Mutex::new(AmpMode::Envelope)),
            audio_device,
            envelope,
            output_level: Arc::new(Mutex::new(OUTPUT_LEVEL)),
            oscillators: oscillators_arc,
            lfos: lfos_arc,
            filter: filter_arc,
            auto_pan,
            tremolo,
            filter_mod,
        }
    }

    pub fn run(&mut self, ui_receiver: Receiver<EventType>) {
        self.stream = Some(self.create_audio_engine());
        println!("Synth run");

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
                        let mut output_level = self.get_output_level_mutex_lock();
                        *output_level = level as f32;
                    }
                    EventType::UpdateEnvelopeAttack(milliseconds) => {
                        let mut envelope = self.get_envelope_mutex_lock();
                        envelope.set_attack_milliseconds(milliseconds.abs() as u32);
                    }
                    EventType::UpdateEnvelopeDecay(milliseconds) => {
                        let mut envelope = self.get_envelope_mutex_lock();
                        envelope.set_decay_milliseconds(milliseconds.abs() as u32);
                    }
                    EventType::UpdateEnvelopeRelease(milliseconds) => {
                        let mut envelope = self.get_envelope_mutex_lock();
                        envelope.set_release_milliseconds(milliseconds.abs() as u32);
                    }
                    EventType::UpdateEnvelopeSustain(milliseconds) => {
                        let mut envelope = self.get_envelope_mutex_lock();
                        envelope.set_sustain_milliseconds(milliseconds.abs() as u32);
                    }
                    EventType::UpdateEnvelopeSustainLevel(level) => {
                        let mut envelope = self.get_envelope_mutex_lock();
                        envelope.set_sustain_level_below_output_level_in_dbfs(level as f32);
                    }
                    EventType::UpdateAmpModeEnvelopeEnabled(is_enabled) => {
                        let mut amp_mode = self.get_amp_mode_mutex_lock();
                        if is_enabled {
                            *amp_mode = AmpMode::Envelope;
                        } else {
                            *amp_mode = AmpMode::Gate;
                        }
                    }
                    EventType::UpdateGateDutyCycle(duty_cycle) => {
                        let mut envelope = self.get_envelope_mutex_lock();
                        envelope.set_gate_duty_cycle(duty_cycle);
                    }
                    EventType::UpdateGateNoteLength(note_length) => {
                        let mut envelope = self.get_envelope_mutex_lock();
                        envelope.set_gate_note_length(note_length.abs() as u32);
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
                    EventType::UpdateAutoPanEnabled(is_enabled) => {
                        let mut auto_pan = self.get_auto_pan_mutex_lock();
                        auto_pan.is_enabled = is_enabled;
                    }
                    EventType::UpdateAutoPanSpeed(speed_hz) => {
                        let mut auto_pan = self.get_auto_pan_mutex_lock();
                        auto_pan.frequency = speed_hz;
                    }
                    EventType::UpdateAutoPanWidth(width) => {
                        let mut auto_pan = self.get_auto_pan_mutex_lock();
                        auto_pan.width = width;
                    }
                    EventType::UpdateTremoloEnabled(is_enabled) => {
                        let mut tremolo = self.get_tremolo_mutex_lock();
                        tremolo.is_enabled = is_enabled;
                    }
                    EventType::UpdateTremoloSpeed(speed_hz) => {
                        let mut tremolo = self.get_tremolo_mutex_lock();
                        tremolo.frequency = speed_hz;
                    }
                    EventType::UpdateTremoloDepth(depth) => {
                        let mut tremolo = self.get_tremolo_mutex_lock();
                        tremolo.width = depth;
                        tremolo.center_frequency = 1.0 - (depth / 2.0);
                    }
                    EventType::UpdateFilterModEnabled(is_enabled) => {
                        let mut filter_mod = self.get_filter_mod_mutex_lock();
                        filter_mod.is_enabled = is_enabled;
                    }
                    EventType::UpdateFilterModSpeed(speed_hz) => {
                        let mut filter_mod = self.get_filter_mod_mutex_lock();
                        filter_mod.frequency = speed_hz;
                    }
                    EventType::UpdateFilterModAmount(amount) => {
                        let mut filter_mod = self.get_filter_mod_mutex_lock();
                        filter_mod.width = amount;
                        filter_mod.center_frequency = 1.0 - (amount / 2.0);
                    }
                    EventType::Start => {
                        self.start();
                    }
                    EventType::Stop => {
                        self.stop();
                    }
                    EventType::Exit => {
                        break;
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

    fn get_filter_mutex_lock(&mut self) -> MutexGuard<Filter> {
        self.filter
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn get_auto_pan_mutex_lock(&mut self) -> MutexGuard<LFOInstance> {
        self.auto_pan
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn get_tremolo_mutex_lock(&mut self) -> MutexGuard<LFOInstance> {
        self.tremolo
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn get_filter_mod_mutex_lock(&mut self) -> MutexGuard<LFOInstance> {
        self.filter_mod
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn get_envelope_mutex_lock(&mut self) -> MutexGuard<Envelope> {
        self.envelope
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn get_output_level_mutex_lock(&mut self) -> MutexGuard<f32> {
        self.output_level
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn get_amp_mode_mutex_lock(&mut self) -> MutexGuard<AmpMode> {
        self.amp_mode
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

        // The sequence is midi note numbers
        // For rests use note 0 - It leaves out c-1 but 8 Hz doesn't do you much good anyway.
        let mut sequencer = Sequencer::new(vec![0, 55, 58, 50, 57, 60, 62, 64, 67, 0]);
        let mut note_frequency = sequencer.next_note_frequency();

        let output_level_arc = self.output_level.clone();
        let amp_mode_arc = self.amp_mode.clone();
        let envelope_arc = self.envelope.clone();
        let oscillators_arc = self.oscillators.clone();
        let filter_arc = self.filter.clone();
        let lfo_arc = self.lfos.clone();
        let auto_pan_arc = self.auto_pan.clone();
        let tremolo_arc = self.tremolo.clone();
        let filter_mod_arc = self.filter_mod.clone();

        let stream = output_device
            .build_output_stream(
                &stream_config,
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

                    let output_level = *output_level_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    let amp_mode = *amp_mode_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    let mut filter = filter_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    let auto_pan = auto_pan_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    let tremolo = tremolo_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    let filter_mod = filter_mod_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    for frame in buffer.chunks_mut(number_of_channels) {
                        let oscillator1_sample = oscillators.get_oscillator1_next_sample(
                            note_frequency,
                            oscillator1_level,
                            None,
                        );
                        let oscillator2_sample = oscillators.get_oscillator2_next_sample(
                            note_frequency,
                            oscillator2_level,
                            None,
                        );
                        let oscillator3_sample = oscillators.get_oscillator3_next_sample(
                            note_frequency,
                            oscillator3_level,
                            None,
                        );
                        let sub_oscillator_sample = oscillators.get_sub_oscillator_next_sample(
                            note_frequency,
                            sub_oscillator_level,
                            None,
                        );

                        let oscillator_sum = oscillator1_sample
                            + oscillator2_sample
                            + oscillator3_sample
                            + sub_oscillator_sample;

                        let oscillator_level_sum = oscillator1_level
                            + oscillator2_level
                            + oscillator3_level
                            + sub_oscillator_level;

                        let level_balanced_oscillator_sum = oscillator_sum / oscillator_level_sum;

                        let filter_mod_value = match filter_mod.is_enabled {
                            true => Some(lfos[LFO_INDEX_FOR_FILTER_MOD].get_next_value(
                                filter_mod.frequency,
                                filter_mod.center_frequency,
                                filter_mod.width,
                            )),
                            false => None,
                        };

                        let filtered_sample =
                            filter.filter_sample(level_balanced_oscillator_sum, filter_mod_value);

                        let (left_panned_sample, right_panned_sample) = match auto_pan.is_enabled {
                            true => {
                                let pan_value = lfos[LFO_INDEX_FOR_AUTO_PAN].get_next_value(
                                    auto_pan.frequency,
                                    auto_pan.center_frequency,
                                    auto_pan.width,
                                );
                                (
                                    filtered_sample * pan_value,
                                    filtered_sample * (1.0 - pan_value),
                                )
                            }
                            false => (filtered_sample, filtered_sample),
                        };

                        let (left_tremolo_sample, right_tremolo_sample) = match tremolo.is_enabled {
                            true => {
                                let tremolo_value = lfos[LFO_INDEX_FOR_TREMOLO].get_next_value(
                                    tremolo.frequency,
                                    tremolo.center_frequency,
                                    tremolo.width,
                                );
                                (
                                    left_panned_sample * tremolo_value,
                                    right_panned_sample * tremolo_value,
                                )
                            }
                            false => (left_panned_sample, right_panned_sample),
                        };

                        let left_sample = left_tremolo_sample;
                        let right_sample = right_tremolo_sample;

                        if amp_mode == AmpMode::Gate {
                            match envelope.gate(output_level) {
                                GateState::On(db_adjustment) => {
                                    frame[0] = left_sample * db_adjustment;
                                    frame[1] = right_sample * db_adjustment;
                                }
                                GateState::Off(db_adjustment) => {
                                    frame[0] = left_sample * db_adjustment;
                                    frame[1] = right_sample * db_adjustment;
                                }
                                GateState::End(db_adjustment) => {
                                    frame[0] = left_sample * db_adjustment;
                                    frame[1] = right_sample * db_adjustment;
                                    note_frequency = sequencer.next_note_frequency();
                                }
                            }
                        } else {
                            match envelope.adsr(output_level) {
                                ADSRState::Playing(db_adjustment) => {
                                    frame[0] = left_sample * db_adjustment;
                                    frame[1] = right_sample * db_adjustment;
                                }
                                ADSRState::Stopped => {
                                    note_frequency = sequencer.next_note_frequency();
                                }
                            }
                        }
                    }
                },
                |err| panic!("an error occurred for the stream: {}", err),
                None,
            )
            .unwrap();

        stream.play().unwrap();

        stream
    }
}
