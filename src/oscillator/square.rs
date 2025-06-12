use crate::oscillator::Oscillator;

const PI: f32 = std::f32::consts::PI;

pub struct Square {
    x_coord: f32,
    x_increment: f32,
    sample_rate: f32,
}

impl Oscillator for Square {
    fn new(sample_rate: f32) -> Self {
        let x_coord = 0.0;
        let x_increment = 1.0;
        let sample_rate = sample_rate;

        Self {
            x_coord,
            x_increment,
            sample_rate,
        }
    }

    fn generate_tone_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let mut y_coord: f32 =
            (tone_frequency * (2.0 * PI) * (self.x_coord / self.sample_rate)).sin();

        let duty_cycle = modulation.unwrap_or_default();

        if y_coord >= 0.0 - duty_cycle {
            y_coord = 1.0;
        } else {
            y_coord = -1.0;
        }

        self.x_coord += self.x_increment;
        y_coord
    }
}
