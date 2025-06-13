pub mod noise;
pub mod ramp;
pub mod saw;
pub mod sine;
pub mod square;
pub mod triangle;

pub trait Oscillator {
    fn new(sample_rate: f32) -> Self;

    fn generate_next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32;
}
