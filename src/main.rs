mod envelope;
mod oscillators;
mod sequencer;
pub mod lfo;
mod device;

use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use cpal::traits::{DeviceTrait, StreamTrait};
use crate::envelope::{Envelope, State};
use crate::oscillators::{WaveShape, Oscillators};
use crate::device::AudioDevice;
use crate::lfo::LFO;
use crate::sequencer::Sequencer;

const OUTPUT_LEVEL: f32 = -10.0; // Sets output level to -10.  Change to any dbfs level you want
const MAIN_LOOP_DURATION_SECONDS: u64 = 60;

fn main() {

    // Initialize the default audio output device for your system
    let audio_device = AudioDevice::new();
    let sample_rate = audio_device.get_sample_rate();

    // Initialize the LFO
    let mut lfo1 = LFO::new(sample_rate);

    // Set up your initial oscillators and set their WaveShape
    // Available WaveShapes: Noise, Ramp, Saw, Sine, Square, SuperSaw, Triangle
    let mut oscillators = Oscillators::new(sample_rate);

    oscillators.set_oscillator1_type(WaveShape::Square);
    oscillators.set_oscillator2_type(WaveShape::Square);
    oscillators.set_oscillator3_type(WaveShape::Square);
    // oscillators.enable_unison(0.03);
    
    // The sequence is midi note numbers
    // For rests use note 0 - It leaves out c-1 but 8 Hz doesn't do you much good anyway. 
    let mut sequencer = Sequencer::new(vec![60, 62, 63, 65, 67, 68, 70, 72]);

    // Initialize the modulation module and define your ADSR Envelope
    let mut modulation = Envelope::new(sample_rate as u32, OUTPUT_LEVEL);
    modulation.set_adsr_attack_milliseconds(50);
    modulation.set_adsr_decay_milliseconds(300);
    modulation.set_adsr_release_milliseconds(200);
    modulation.set_adsr_sustain_length_milliseconds(300);
    modulation.set_adsr_sustain_level(OUTPUT_LEVEL - 6.0);
    
    
    // Build the output stream that will be sent through CoreAudio to your selected device
    let mut note_frequency = sequencer.next_note_frequency();
    let stream_config = audio_device.get_stream_config();
    let output_device = audio_device.get_output_device();
    let number_of_channels = audio_device.get_number_of_channels();

    let oscillators_arc = Arc::new(Mutex::new(oscillators));

    let stream = output_device
        .build_output_stream(
            &stream_config,
            move |buffer: &mut [f32], _: &cpal::OutputCallbackInfo| {

                let mut oscillator = oscillators_arc.lock().unwrap_or_else(|poisoned| {
                    poisoned.into_inner()
                });

                for frame in buffer.chunks_mut(number_of_channels) {

                    let lfo1_value = lfo1.get_next_value(3.0, 0.7, 0.3);

                    let oscillator1_sample = oscillator.get_oscillator1_next_sample(note_frequency, 1.0, None);
                    let oscillator2_sample = oscillator.get_oscillator2_next_sample(note_frequency, 1.0, Some(lfo1_value));
                    let oscillator3_sample = oscillator.get_oscillator3_next_sample(note_frequency, 1.0, None);

                    let (left_sample, right_sample) = if oscillator.is_unison() {
                        let left = oscillator1_sample + oscillator2_sample + oscillator3_sample;
                        let right = oscillator1_sample + oscillator2_sample + oscillator3_sample;
                        (left, right)
                    } else {
                        // If not using a unison spread, the waves will be the same and the peaks will get very loud
                        // Divide by 2 to reduce them by 6 dbfs to compensate some
                        let left = (oscillator1_sample + oscillator2_sample + oscillator3_sample)/2.0;
                        let right = (oscillator1_sample + oscillator2_sample + oscillator3_sample)/2.0;
                        (left, right)
                    };

                    match modulation.envelope(OUTPUT_LEVEL) {
                        State::Playing(db_adjustment) => {
                            frame[0] = left_sample * db_adjustment;
                            frame[1] = right_sample * db_adjustment;
                        }
                        State::Stopped => {
                            note_frequency = sequencer.next_note_frequency();
                        },
                    }

                }

            },
            |err| panic!("an error occurred for the stream: {}", err),
            None,
        )
        .unwrap();
    
        stream.play().unwrap();

        sleep(Duration::from_secs(MAIN_LOOP_DURATION_SECONDS));

}