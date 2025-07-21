use crate::synth::oscillators::GenerateSamples;

const PI: f32 = std::f32::consts::PI;
const DEFAULT_X_COORDINATE: f32 = 0.0;
const DEFAULT_X_INCREMENT: f32 = 1.0;
const VOICE_FREQUENCY_SPREAD: [f32; 5] = [0.98, 0.99, 1.0, 1.01, 1.02];

pub struct SuperSaw {
    x_coordinate: f32,
    x_increment: f32,
    sample_rate: f32,
}

impl SuperSaw {
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

impl GenerateSamples for SuperSaw {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let mut voice_samples: Vec<f32> = vec![];

        for frequency_offset in VOICE_FREQUENCY_SPREAD {
            voice_samples.push(self.single_saw_sample(
                tone_frequency * frequency_offset,
                self.x_coordinate,
                modulation,
            ));
        }

        self.x_coordinate += self.x_increment;

        voice_samples.iter().sum::<f32>() / 2.0
    }

    fn reset(&mut self) {
        self.x_coordinate = DEFAULT_X_COORDINATE;
        self.x_increment = DEFAULT_X_INCREMENT;
    }
}

impl SuperSaw {
    fn single_saw_sample(
        &mut self,
        tone_frequency: f32,
        x_coordinate: f32,
        modulation: Option<f32>,
    ) -> f32 {
        let new_frequency = tone_frequency * modulation.unwrap_or(1.0);

        let y_coordinate: f32 = (-2.0 / PI)
            * (1.0f32 / (new_frequency * PI * (x_coordinate / self.sample_rate)).tan()).atan();
        y_coordinate
    }
}
