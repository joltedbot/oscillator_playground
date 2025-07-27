use crate::synth::oscillators::GenerateSamples;
use crate::synth::oscillators::sine::Sine;
use crate::synth::oscillators::saw::Saw;

const DEFAULT_RATIO: f32 = 1.0;
const DEFAULT_MODULATION_AMOUNT: f32 = 3.0;
const AM_SLIDER_ADJUSTMENT_FACTOR: f32 = 10.0;

pub struct AM {
    carrier: Box<dyn GenerateSamples + Send + Sync>,
    modulator: Box<dyn GenerateSamples + Send + Sync>,
    modulation_amount: f32,
    ratio: f32,
}

impl AM {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            carrier: Box::new(Sine::new(sample_rate)),
            modulator: Box::new(Sine::new(sample_rate)),
            modulation_amount: DEFAULT_MODULATION_AMOUNT,
            ratio: DEFAULT_RATIO,
        }
    }
}

impl GenerateSamples for AM {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let modulation = modulation.unwrap_or(1.0);
        let modulator = self
            .modulator
            .next_sample(self.modulation_amount * modulation, None);
        self.carrier.next_sample(tone_frequency, None) * modulator
    }

    fn set_shape_specific_parameter(&mut self, parameter: f32) {
        self.modulation_amount = parameter/AM_SLIDER_ADJUSTMENT_FACTOR;
    }

    fn reset(&mut self) {
        self.carrier.reset();
        self.modulator.reset();
    }
}
