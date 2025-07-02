const RADS_PER_CYCLE: f32 = 2.0 * std::f32::consts::PI;

pub struct LFO {
    pub phase: f32,
    pub phase_increment: f32,
    pub sample_rate: f32,
}

impl LFO {
    pub fn new(sample_rate: f32) -> Self {
        let phase: f32 = 0.0;
        let seconds_per_sample = 1.0 / sample_rate;
        let phase_increment = RADS_PER_CYCLE * seconds_per_sample;

        Self {
            phase,
            sample_rate,
            phase_increment,
        }
    }

    pub fn get_next_value(&mut self, lfo_frequency: f32, center_value: f32, spread: f32) -> f32 {
        let phase_increment = self.phase_increment * lfo_frequency;

        self.phase += phase_increment;
        if self.phase >= RADS_PER_CYCLE {
            self.phase = 0.0;
        }
        let wave_position = self.phase.sin();

        center_value + (wave_position * (spread / 2.0))
    }
}
