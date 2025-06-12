use crate::oscillator::Oscillator;

const RADS_PER_CYCLE: f32 = 2.0 * std::f32::consts::PI;

pub struct Sine {
    pub phase: f32,
    pub phase_increment: f32,
    pub sample_rate: f32,
}

impl Oscillator for Sine {
    fn new(sample_rate: f32) -> Self {
        let phase: f32 = 0.0;
        let seconds_per_sample = 1.0 / sample_rate;
        let phase_increment = RADS_PER_CYCLE * seconds_per_sample;

        Self {
            phase,
            sample_rate,
            phase_increment,
        }
    }

    fn generate_tone_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let phase_modulation = match modulation {
            Some(modulation) => modulation,
            None => self.phase_increment * tone_frequency,
        };

        self.phase += phase_modulation;
        if self.phase >= RADS_PER_CYCLE {
            self.phase = 0.0;
        }
        self.phase.sin()
    }
}
