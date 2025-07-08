use crate::events::EventType;
use crate::synth::dynamics::Dynamics;
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AmpMode {
    Gate,
    Envelope,
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

        Self {
            stream: None,
            amp_mode: Arc::new(Mutex::new(AmpMode::Envelope)),
            audio_device,
            envelope,
            output_level: Arc::new(Mutex::new(OUTPUT_LEVEL)),
            oscillators: oscillators_arc,
            lfos: lfos_arc,
            filter: filter_arc,
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
                        filter.set_resonance(level as f32);
                    }
                    EventType::UpdateFilterNumberOfPoles(number_of_poles) => {
                        let mut filter = self.get_filter_mutex_lock();
                        filter.set_number_of_poles(number_of_poles);
                    }
                    EventType::ResyncOscillators => {
                        let mut oscillators = self.get_oscillators_mutex_lock();
                        oscillators.reset();
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

        let dynamics = Dynamics::new();

        // The sequence is midi note numbers
        // For rests use note 0 - It leaves out c-1 but 8 Hz doesn't do you much good anyway.
        let mut sequencer = Sequencer::new(vec![0, 65, 68, 70, 77, 80, 82, 84, 87, 0]);
        let mut note_frequency = sequencer.next_note_frequency();

        let output_level_arc = self.output_level.clone();
        let amp_mode_arc = self.amp_mode.clone();
        let envelope_arc = self.envelope.clone();
        let oscillators_arc = self.oscillators.clone();
        let filter_arc = self.filter.clone();

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
                        let filtered_sample = filter.filter_sample(level_balanced_oscillator_sum);

                        let left_sample = filtered_sample;
                        let right_sample = filtered_sample;

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
