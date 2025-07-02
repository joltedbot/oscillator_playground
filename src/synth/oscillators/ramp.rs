use crate::synth::oscillators::GenerateSamples;

const PI: f32 = std::f32::consts::PI;

pub struct Ramp {
    x_coord: f32,
    x_increment: f32,
    sample_rate: f32,
}

impl Ramp {
    pub fn new(sample_rate: f32) -> Self {
        let x_coord = 0.0;
        let x_increment = 1.0;

        Self {
            x_coord,
            x_increment,
            sample_rate,
        }
    }
}

impl GenerateSamples for Ramp {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let y_coord: f32 = (2.0 / PI)
            * modulation.unwrap_or(1.0)
            * (1.0f32 / (tone_frequency * PI * (self.x_coord / self.sample_rate)).tan()).atan();

        self.x_coord += self.x_increment;
        y_coord
    }
}
