use crate::synth::oscillators::GenerateSamples;

const LOWEST_FREQUENCY_TO_SUB: f32 = 33.0;

pub struct Sub {
    oscillator: Box<dyn GenerateSamples + Send + Sync>,
}

impl Sub {
    pub fn new(oscillator: Box<dyn GenerateSamples + Send + Sync>) -> Self {
        Self { oscillator }
    }
}

impl GenerateSamples for Sub {
    fn next_sample(&mut self, mut tone_frequency: f32, modulation: Option<f32>) -> f32 {
        if tone_frequency > LOWEST_FREQUENCY_TO_SUB {
            tone_frequency *= 0.5;
        }
        self.oscillator.next_sample(tone_frequency, modulation)
    }

    fn set_shape_specific_parameters(&mut self, parameters: (f32, f32)) {
        self.oscillator.set_shape_specific_parameters(parameters);
    }

    fn reset(&mut self) {
        self.oscillator.reset();
    }
}
