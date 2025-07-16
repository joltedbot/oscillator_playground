use crate::synth::oscillators::GenerateSamples;

const PI: f32 = std::f32::consts::PI;
const DEFAULT_X_COORDINATE: f32 = 0.0;
const DEFAULT_X_INCREMENT: f32 = 1.0;

pub struct Triangle {
    x_coordinate: f32,
    x_increment: f32,
    sample_rate: f32,
}

impl Triangle {
    pub fn new(sample_rate: f32) -> Self {
        let x_coordinate = DEFAULT_X_COORDINATE;
        let x_increment = DEFAULT_X_INCREMENT;

        Self {
            x_coordinate,
            x_increment,
            sample_rate,
        }
    }
}

impl GenerateSamples for Triangle {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let y_coordinate: f32 = 2.0 / PI
            * modulation.unwrap_or(1.0)
            * (tone_frequency * (2.0 * PI) * (self.x_coordinate / self.sample_rate))
                .sin()
                .asin();

        self.x_coordinate += self.x_increment;
        y_coordinate
    }

    fn reset(&mut self) {
        self.x_coordinate = DEFAULT_X_COORDINATE;
        self.x_increment = DEFAULT_X_INCREMENT;
    }
}
