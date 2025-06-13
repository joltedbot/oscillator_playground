use crate::oscillator::Oscillator;

pub struct Noise {}

impl Oscillator for Noise {
    fn new(sample_rate: f32) -> Self {
        Self {}
    }

    fn generate_next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        if tone_frequency == 0.0 {
            return 0.0;
        }
        rand::random_range(-1.0..=1.0)
    }
}
