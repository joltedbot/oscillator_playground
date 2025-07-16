use crate::synth::oscillators::GenerateSamples;

const PI: f32 = std::f32::consts::PI;
const DEFAULT_X_COORDINATE: f32 = 0.0;
const DEFAULT_X_INCREMENT: f32 = 1.0;

pub struct Square {
    x_coordinate: f32,
    x_increment: f32,
    sample_rate: f32,
}

impl Square {
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

impl GenerateSamples for Square {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let mut y_coordinate: f32 =
            (tone_frequency * (2.0 * PI) * (self.x_coordinate / self.sample_rate)).sin();

        let duty_cycle = if tone_frequency == 0.0 {
            0.0
        } else {
            modulation.unwrap_or_default()
        };

        if y_coordinate >= 0.0 - duty_cycle {
            y_coordinate = 1.0;
        } else {
            y_coordinate = -1.0;
        }

        self.x_coordinate += self.x_increment;
        y_coordinate
    }

    fn reset(&mut self) {
        self.x_coordinate = DEFAULT_X_COORDINATE;
        self.x_increment = DEFAULT_X_INCREMENT;
    }
}
