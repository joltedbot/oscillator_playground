use crate::synth::oscillators::{GenerateSamples};
use crate::synth::oscillators::sine::Sine;

pub struct FM {
    carrier: Box<dyn GenerateSamples + Send + Sync>,
    modulator: Box<dyn GenerateSamples + Send + Sync>,
}

impl FM {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            carrier: Box::new(Sine::new(sample_rate)),
            modulator: Box::new(Sine::new(sample_rate)),
        }
    }
}

impl GenerateSamples for FM {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let external_modulation = modulation.unwrap_or(1.0);
        let modulator = self.modulator.next_sample(tone_frequency * external_modulation, None);
        self.carrier.next_sample(tone_frequency * modulator, None)
    }

    fn reset(&mut self) {
        self.carrier.reset();
        self.modulator.reset();
    }
}
