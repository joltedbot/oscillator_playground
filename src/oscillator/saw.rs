use crate::oscillator::Oscillator;

const PI: f32 = std::f32::consts::PI;

pub struct Saw {
    x_coord: f32,
    x_increment: f32,
    sample_rate: f32,
    tone_frequency: f32,

}

impl Oscillator for  Saw { 
    fn new(sample_rate: f32, tone_frequency: u32) -> Self {
        let x_coord = 0.0;
        let x_increment = 1.0;

        Self {
            x_coord,
            x_increment,
            tone_frequency: tone_frequency as f32,
            sample_rate,
        }
    }

    fn generate_tone_sample(&mut self, modulation: Option<f32>) -> f32 {
        let y_coord: f32 = (-2.0 / PI) * (1.0f32 / (self.tone_frequency * PI * (self.x_coord/self.sample_rate)).tan())
            .atan();

        self.x_coord += self.x_increment + modulation.unwrap_or(1.0);
        y_coord
    }
}
