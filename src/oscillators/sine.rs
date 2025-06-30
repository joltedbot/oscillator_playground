use crate::oscillators::GenerateSamples;

const RADS_PER_CYCLE: f32 = 2.0 * std::f32::consts::PI;


pub struct Sine {
    pub phase: f32,
    pub phase_increment: f32,
}

impl Sine {
    pub fn new(sample_rate: f32) -> Self {
        let phase: f32 = 0.0;
        let seconds_per_sample = 1.0 / sample_rate;
        let phase_increment = RADS_PER_CYCLE * seconds_per_sample;

        Self {
            phase,
            phase_increment,
        }
    }
}

impl GenerateSamples for Sine {
    
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        
        self.phase += self.phase_increment * (tone_frequency * modulation.unwrap_or(1.0));
    
        if self.phase >= RADS_PER_CYCLE {
            self.phase = 0.0;
        }
        
        self.phase.sin()
    }
}
