use crate::oscillator::Oscillator;

const PI: f32 = std::f32::consts::PI;

pub struct Triangle {
    x_coord: f32,
    x_increment: f32,
    sample_rate: f32,
    tone_frequency: f32,
}

impl Oscillator for Triangle {
    fn new(sample_rate: f32, tone_frequency: u32) -> Self {
        let x_coord = 0.0;
        let x_increment = 1.0;
        let sample_rate = sample_rate;

        Self {
            x_coord,
            x_increment,
            sample_rate,
            tone_frequency: tone_frequency as f32,
        }
    }

    fn generate_tone_sample(&mut self, modulation: Option<f32>) -> f32 {
        let y_coord: f32 = 2.0 / PI
            * ((self.tone_frequency * (2.0 * PI) * (self.x_coord / self.sample_rate)).sin()).asin();

        self.x_coord += self.x_increment + modulation.unwrap_or(1.0);
        y_coord
    }
}
