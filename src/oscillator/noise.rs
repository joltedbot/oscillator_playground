use crate::oscillator::Oscillator;

pub struct Noise {}

impl Oscillator for Noise {
    fn new(sample_rate: f32) -> Self {
        Self {}
    }

    fn generate_tone_sample(&mut self, _tone_frequency: f32, modulation: Option<f32>) -> f32 {
        rand::random_range(-1.0..=1.0)
    }
}
