use std::f32::consts::PI;
use cpal::BufferSize::Default;

const RESONANCE_Q: f32 = 0.0; // Butterworth
const DEFAULT_CUTOFF_FREQUENCY: f32 = 20000.1;
const FILTER_MAX_CUTOFF_BEFORE_BYPASSING: f32 = 19999.99;

#[derive(Default)]
pub enum Poles {
    One,
    #[default]
    Two,
    Three
}

#[derive(Default)]
pub struct Filter {
    sample_rate: f32,
    cutoff_frequency: f32,
    resonance_q: f32,
    number_of_poles: Poles,
    pole1_buffer_0: f32,
    pole1_buffer_1: f32,
    pole2_buffer_0: f32,
    pole2_buffer_1: f32,
    pole3_buffer_0: f32,
    pole3_buffer_1: f32,
}

impl Filter {
    pub fn new(sample_rate: f32) -> Self {

        Self{
            sample_rate,
            cutoff_frequency: DEFAULT_CUTOFF_FREQUENCY,
            resonance_q: RESONANCE_Q,
            ..Self::default()
        }
    }
    
    pub fn filter_sample(&mut self, sample: f32) -> f32 {
        if self.cutoff_frequency  > FILTER_MAX_CUTOFF_BEFORE_BYPASSING {
            return sample 
        }

        let normalized_frequency = get_normalized_frequency(self.cutoff_frequency, self.sample_rate);
        let feedback = get_feedback_amount(self.resonance_q, normalized_frequency);
        self.filter_sample_pole1(sample, normalized_frequency, feedback);
        
        match self.number_of_poles {
            Poles::One => {
                self.pole1_buffer_1
            },
            Poles::Two => {
                self.filter_sample_pole2(normalized_frequency, feedback);
                self.pole2_buffer_1
            },
            Poles::Three => {
                self.filter_sample_pole2(normalized_frequency, feedback);
                self.filter_sample_pole3(normalized_frequency, feedback);
                self.pole3_buffer_1
            },
        }
        
    }

    fn filter_sample_pole1(&mut self, sample: f32, normalized_frequency: f32, feedback: f32) {
        let high_pass = sample - self.pole1_buffer_0;
        let band_pass = self.pole1_buffer_0 - self.pole1_buffer_1;
        self.pole1_buffer_0 = self.pole1_buffer_0 + normalized_frequency * (high_pass + feedback * band_pass);
        self.pole1_buffer_1 = self.pole1_buffer_1 + normalized_frequency * (self.pole1_buffer_0 - self.pole1_buffer_1);
    }

    fn filter_sample_pole2(&mut self, normalized_frequency: f32, feedback: f32) {
        let high_pass = self.pole1_buffer_1 - self.pole2_buffer_0;
        let band_pass = self.pole2_buffer_0 - self.pole2_buffer_1;
        self.pole2_buffer_0 = self.pole2_buffer_0 + normalized_frequency * (high_pass + feedback * band_pass);
        self.pole2_buffer_1 = self.pole2_buffer_1 + normalized_frequency * (self.pole2_buffer_0 - self.pole2_buffer_1);
    }

    fn filter_sample_pole3(&mut self,normalized_frequency: f32, feedback: f32) {
        let high_pass = self.pole2_buffer_1 - self.pole3_buffer_0;
        let band_pass = self.pole3_buffer_0 - self.pole3_buffer_1;
        self.pole3_buffer_0 = self.pole3_buffer_0 + normalized_frequency * (high_pass + feedback * band_pass);
        self.pole3_buffer_1 = self.pole3_buffer_1 + normalized_frequency * (self.pole3_buffer_0 - self.pole3_buffer_1);
    }

    pub fn set_cutoff_frequency(&mut self, cutoff_frequency: f32) {
        self.cutoff_frequency = cutoff_frequency;
    }
    
    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance_q = resonance;
    }

    pub fn set_number_of_poles(&mut self, number_of_poles: i32) {
        self.number_of_poles = get_number_of_poles_from_integer(number_of_poles);
    }
    
}

fn get_normalized_frequency(cutoff_frequency: f32, sample_rate: f32) -> f32 {
    PI*cutoff_frequency/sample_rate 
}

fn get_feedback_amount(resonance_q: f32, normalized_frequency: f32) -> f32 {
    resonance_q + resonance_q/(1.0 - normalized_frequency)
}


fn get_number_of_poles_from_integer(number_of_poles_word: i32) -> Poles {
    match number_of_poles_word {
        1 => Poles::One,
        2 => Poles::Two,
        3 => Poles::Three,
        _ => Poles::default(),
    }
}