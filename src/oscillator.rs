pub mod noise;
pub mod ramp;
pub mod saw;
pub mod sine;
pub mod square;
pub mod triangle;

pub trait Oscillator {
    fn new(sample_rate: f32, tone_frequency: u32) -> Self;

    fn generate_tone_sample(&mut self, modulation: Option<f32>) -> f32;
}
