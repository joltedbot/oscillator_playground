use crate::synth::oscillators::GenerateSamples;

pub struct Noise {}

impl Noise {
    pub fn new() -> Self {
        Self {}
    }
}

impl GenerateSamples for Noise {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        if tone_frequency == 0.0 {
            return 0.0;
        }
        rand::random_range(-1.0..=1.0) * modulation.unwrap_or(1.0)
    }

    fn set_shape_specific_parameters(&mut self, _parameter: (f32, f32)) {}

    fn reset(&mut self) {}
}
