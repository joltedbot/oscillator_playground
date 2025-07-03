use std::f32::consts::PI;
use cpal::BufferSize::Default;

const RESONANCE_Q: f32 = 0.0; // Butterworth
const DEFAULT_CUTOFF_FREQUENCY: f32 = 440.0;

#[derive(Default)]
pub struct Filter {
    sample_rate: f32,
    cutoff_frequency: f32,
    resonance_q: f32,
    sample_buffer_1: f32,
    sample_buffer_2: f32,
}

impl Filter {
    pub fn new(sample_rate: f32) -> Self {
        
        Self{
            sample_rate,
            cutoff_frequency: DEFAULT_CUTOFF_FREQUENCY,
            resonance_q: RESONANCE_Q,
            sample_buffer_1: 0.0,
            sample_buffer_2: 0.0,
        }
    }
    
    pub fn filter_sample(&mut self, sample: f32) -> f32 {
        let normalized_frequency = get_normalized_frequency(self.cutoff_frequency, self.sample_rate);
        let feedback = get_feedback_amount(self.resonance_q, normalized_frequency);
        let high_pass = sample - self.sample_buffer_1;
        let band_pass = self.sample_buffer_1 - self.sample_buffer_2;
        self.sample_buffer_1 = self.sample_buffer_1 + normalized_frequency * (high_pass + feedback * band_pass);
        self.sample_buffer_2 = self.sample_buffer_2 + normalized_frequency * (self.sample_buffer_1 - self.sample_buffer_2);

        self.sample_buffer_2
    }
    
    pub fn set_cutoff_frequency(&mut self, cutoff_frequency: f32) {
        self.cutoff_frequency = cutoff_frequency;
    }
    
    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance_q = resonance;
    }
    
}

fn get_normalized_frequency(cutoff_frequency: f32, sample_rate: f32) -> f32 {
    2.0*(PI*cutoff_frequency/sample_rate).sin()
}

fn get_feedback_amount(resonance_q: f32, normalized_frequency: f32) -> f32 {
    resonance_q + resonance_q/(1.0 - normalized_frequency)
}


