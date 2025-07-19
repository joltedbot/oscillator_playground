use crate::synth::oscillators::GenerateSamples;

pub struct LFO {
    oscillator: Box<dyn GenerateSamples + Send + Sync>,
}

impl LFO {
    pub fn new(oscillator: Box<dyn GenerateSamples + Send + Sync>) -> Self {
        Self { oscillator }
    }

    pub fn get_next_value(&mut self, lfo_frequency: f32, center_value: f32, spread: f32) -> f32 {
        let wave_position = self.oscillator.next_sample(lfo_frequency, None);

        center_value + (wave_position * (spread / 2.0))
    }

    pub fn reset(&mut self) {
        self.oscillator.reset();
    }
}
