use crate::oscillators::GenerateSamples;

const PI: f32 = std::f32::consts::PI;

const VOICE_FREQUENCY_SPREAD: [f32; 5] = [0.95, 0.97, 1.0, 1.03, 1.05];


pub struct SuperSaw {
    x_coord: f32,
    x_increment: f32,
    sample_rate: f32,
}

impl SuperSaw {
    pub fn new(sample_rate: f32) -> Self {
        let x_coord = 0.0;
        let x_increment = 1.0;

        Self {
            x_coord,
            x_increment,
            sample_rate,
        }
    }
}

impl GenerateSamples for SuperSaw {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let mut voice_samples: Vec<f32> = vec![];
        
        for frequency_offset in VOICE_FREQUENCY_SPREAD {
            voice_samples.push(self.single_saw_sample(tone_frequency * frequency_offset, self.x_coord, modulation));
        }

        self.x_coord += self.x_increment;

        voice_samples.iter().sum::<f32>() / 4.0
    }
    
}

impl SuperSaw {
    
    fn single_saw_sample(&mut self, tone_frequency: f32, x_coord: f32, modulation: Option<f32>) -> f32 {
        let y_coord: f32 = (-2.0 / PI) * modulation.unwrap_or(1.0)
            * (1.0f32 / (tone_frequency * PI * (x_coord / self.sample_rate)).tan())
            .atan();
        y_coord
    }
    
}