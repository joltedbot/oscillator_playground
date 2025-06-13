#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
mod modulation;
mod oscillator;
mod sequencer;
pub mod lfo;

use crate::oscillator::{
    noise::Noise, ramp::Ramp, saw::Saw, sine::Sine, square::Square, triangle::Triangle, Oscillator
};
use crate::modulation::{Modulation, State};
use crate::lfo::LFO;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{default_host, StreamConfig};
use sequencer::Sequencer;


const TONE_FREQUENCY: f32 = 110.0; // Sets the frequency of the tone. It is in Hertz. Change to whatever you want
const UNISON_SPREAD: f32 = 0.5;

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
    let mut saw = Saw::new(default_sample_rate as f32);
    let mut ramp = Ramp::new(default_sample_rate as f32);
    let mut triangle = Triangle::new(default_sample_rate as f32);
    let mut sine = Sine::new(default_sample_rate  as f32);
    let mut noise = Noise::new(default_sample_rate as f32);
    
    let mut square = Square::new(default_sample_rate as f32);
    let mut square2 = Square::new(default_sample_rate as f32);
    let mut square3 = Square::new(default_sample_rate as f32);

    let mut lfo = LFO::new(default_sample_rate as f32);


    // Initialize the modulation module if you want to use pwm modulation.
    let mut modulation = Modulation::new(default_sample_rate, OUTPUT_LEVEL);
    modulation.set_attack_milliseconds(50);
    modulation.set_decay_milliseconds(300);
    modulation.set_release_milliseconds(200);
    modulation.set_sustain_length_milliseconds(100);
    modulation.set_sustain_level(OUTPUT_LEVEL - 6.0);

    let buffer_delay_count = 4400;
    let mut count: u64 = 0;
    let mut sample_delay_buffer: Vec<f32> = Vec::with_capacity(buffer_delay_count);
    let mut buffered_sample: f32 = 0.0;
    
    // The sequence is midi note numbers
    // For rests use note 0 - It leaves out c-1 but 8hz doesn't do you much good anyway. 
    let mut sequencer = Sequencer::new(vec![55, 57, 58, 0, 62, 63, 65, 67]);
    let mut note_frequency = sequencer.next_note_frequency();
    
    // Build the output stream that will be sent through CoreAudio to your selected device
    let stream = output_device
        .build_output_stream(
            &stream_config,
            move |buffer: &mut [f32], _: &cpal::OutputCallbackInfo| {

                // This is the main action loop that takes in a buffer and breaks it into fames
                // One frame is an array of 1 sample from each channel on the selected device
                for frame in buffer.chunks_mut(number_of_channels as usize) {
                    // Set up PWM modulation.  Pass "Some(duty_cycle) to the generate_next_sample function
                    let pwm_amount_percentage = 0.4; // i.e., the % to change the duty cycle during modulation
                    let pwm = modulation.pwm(pwm_amount_percentage);

                    let lfo_value = lfo.generate_next_sample(50.0, 0.1, 0.5);

                    // Gets the next sample for the selected wave form
                    let sine_sample = sine.generate_next_sample(note_frequency, None);
                     let saw_sample = saw.generate_next_sample(note_frequency, None);
                    let ramp_sample = ramp.generate_next_sample(note_frequency, Some(pwm));
                    let mut triangle_sample = triangle.generate_next_sample(note_frequency, None);
                    let noise_sample = noise.generate_next_sample(note_frequency, None);

                    let square_sample = square.generate_next_sample(note_frequency, None);
                    let square_sample2 = square2.generate_next_sample(note_frequency + UNISON_SPREAD, Some(pwm));
                    let square_sample3 = square3.generate_next_sample(note_frequency - UNISON_SPREAD, Some(lfo_value));
                    
                    // Multiple shape adding plus a sample delay
                    /*
                    if count >= buffer_delay_count as u64 {
                        buffered_sample = sample_delay_buffer.pop().unwrap();
                    } 

                    sample_delay_buffer.insert(0, saw_sample);
                    count += 1;

                    let mut left_sample = square_sample + buffered_sample;
                    let mut right_sample = square_sample + buffered_sample;
                    */
                    
                    // Unison 
                   //  let mut left_sample = square_sample + square_sample2 + square_sample3;
                    // let mut right_sample = square_sample + square_sample2 + square_sample3;

                    // Set the left and right out to one of the _sample variables above
                    // This is also where you can mess with the samples. * -1.0 to invert the phase, add, subtract,
                    // multiply wave forms etc...
                       let mut left_sample = sine_sample;
                       let mut right_sample = sine_sample;

                    // Send the sample to the left and right channels and multiply by the output level adjustment form above
                    // Left is set to Channel 1 (0 here) and right to Channel 2 (1 here). You can change just -1 from the
                    // normal channel number for the interface
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




    loop {
        // Loops forever.  Use Ctrl-C in the terminal to exit.
     }
}

fn get_output_level_adjustment_factor(output_level: f32) -> f32 {
    10.0_f32.powf(output_level / 20.0)
}
