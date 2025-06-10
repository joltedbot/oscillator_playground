#![allow(dead_code, unused_variables)]
mod modulation;
mod oscillator;

use crate::oscillator::{
    Oscillator, noise::Noise, ramp::Ramp, saw::Saw, sine::Sine, square::Square, triangle::Triangle,
};
use std::process::exit;

use crate::modulation::{Modulation, State};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{StreamConfig, default_host};

const TONE_FREQUENCY: u32 = 350; // Sets the frequency of the tone. It is in Hertz. Change to whatever you want
const OUTPUT_LEVEL: f32 = -10.0; // Sets output level to -18.  Change to any dbfs level you want

fn main() {
    // This will grab the device that is currently selected to play audio in macOS.  Focusrite, Speakers, etc...
    let host = default_host();
    let output_device = host.default_output_device().unwrap();

    // Grabs the supported configuration of whatever device is selected
    let stream_config: StreamConfig = output_device.default_output_config().unwrap().into();
    let default_sample_rate = stream_config.sample_rate.0;
    let number_of_channels = stream_config.channels;

    //  Adjust the volume of the output.  To change set the OUTPUT_LEVEL or set to any arbitrary dbfs value
    let output_level = get_output_level_adjustment_factor(OUTPUT_LEVEL);

    // Initialize the oscillators you want to use.
    // let mut saw = Saw::new(default_sample_rate, TONE_FREQUENCY);
    let mut ramp = Ramp::new(default_sample_rate as f32, TONE_FREQUENCY);
    // let mut triangle = Triangle::new(default_sample_rate, TONE_FREQUENCY);
    // let mut sine = Sine::new(default_sample_rate  as f32, TONE_FREQUENCY);
    let mut square = Square::new(default_sample_rate as f32, TONE_FREQUENCY);
    // let mut noise = Noise::new(default_sample_rate, TONE_FREQUENCY);

    // Initialize the modulation module if you want to use pwm modulation.
    let mut modulation = Modulation::new(default_sample_rate, OUTPUT_LEVEL);
    modulation.set_attack_milliseconds(2000);
    modulation.set_decay_milliseconds(2000);
    modulation.set_release_milliseconds(2000);
    modulation.set_sustain_length_milliseconds(2000);
    modulation.set_sustain_level(OUTPUT_LEVEL - 9.0);

    // Build the output stream that will be sent through CoreAudio to your selected device
    let stream = output_device
        .build_output_stream(
            &stream_config,
            move |buffer: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // This is the main action loop that takes in a buffer and breaks it into fames
                // One frame is an array of 1 sample from each channel on the selected device
                for frame in buffer.chunks_mut(number_of_channels as usize) {
                    // Set up PWM modulation.  Pass "Some(duty_cycle) to the generate
                    let pwm_amount_percentage = -0.5; // ie. the % to change the duty cycle during modulation
                    let pwm = modulation.pwm(pwm_amount_percentage);

                    // Gets the next sample for the selected wave form
                    //let sine_sample = sine.generate_tone_sample(None);
                    // let saw_sample = saw.generate_tone_sample(Some(pwm));
                    let ramp_sample = ramp.generate_tone_sample(Some(pwm));
                    // let triangle_sample = triangle.generate_tone_sample(Some(pwm));
                    let square_sample = square.generate_tone_sample(None);
                    // let noise_sample = noise.generate_tone_sample(None);

                    // Set the left and right out to one of the _sample variables above
                    // This is also where you can mess with the samples. * -1.0 to invert the phase, add, subtract,
                    // multiply wave forms etc...
                    let left_sample = (square_sample + ramp_sample) / 2.0;
                    let right_sample = (square_sample + ramp_sample) / 2.0;

                    // Send the sample to the left and right channels and multiply by the output level adjustment form above
                    // Left is set to Channel 1 (0 here) and right to Channel 2 (1 here). You can change just -1 from the
                    // normal channel number for the interface
                    match modulation.envelope(OUTPUT_LEVEL) {
                        State::Playing(db_adjustment) => {
                            frame[0] = left_sample * db_adjustment;
                            frame[1] = right_sample * db_adjustment;
                        }
                        State::Stopped => exit(0),
                    }
                }
            },
            |err| panic!("an error occurred for the stream: {}", err),
            None,
        )
        .unwrap();

    stream.play().unwrap();

    println!("Playing. . .");

    loop {
        // Loops forever.  Use Ctrl-C in the terminal to exit.
    }
}

fn get_output_level_adjustment_factor(output_level: f32) -> f32 {
    10.0_f32.powf(output_level / 20.0)
}
