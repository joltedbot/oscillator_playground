use crate::synth::oscillators::GenerateSamples;

const PI: f32 = std::f32::consts::PI;

pub struct Triangle {
    x_coord: f32,
    x_increment: f32,
    sample_rate: f32,
}

impl Triangle {
    pub fn new(sample_rate: f32) -> Self {
        let x_coord = 0.0;
        let x_increment = 1.0;
        let sample_rate = sample_rate;

        Self {
            x_coord,
            x_increment,
            sample_rate,
        }
    }
}

impl GenerateSamples for Triangle {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let y_coord: f32 = 2.0 / PI
            * modulation.unwrap_or(1.0)
            * (tone_frequency * (2.0 * PI) * (self.x_coord / self.sample_rate))
                .sin()
                .asin();

        self.x_coord += self.x_increment;
        y_coord
    }
}
