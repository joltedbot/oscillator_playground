pub mod noise;
pub mod ramp;
pub mod saw;
pub mod sine;
pub mod square;
pub mod triangle;
pub mod super_saw;


use noise::Noise;
use ramp::Ramp;
use saw::Saw;
pub(crate) use sine::Sine;
use square::Square;
use super_saw::SuperSaw;
use triangle::Triangle;

pub trait GenerateSamples {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32;
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaveShape {
    Noise,
    Ramp,
    Saw,
    Sine,
    Square,
    SuperSaw,
    Triangle,
}

pub struct Oscillators {
    sample_rate: f32,
    oscillator1: Box<dyn GenerateSamples + Send + Sync>,
    oscillator2: Box<dyn GenerateSamples + Send + Sync>,
    oscillator3: Box<dyn GenerateSamples + Send + Sync>,
    is_unison: bool,
    unison_frequency_offset: f32,
}

impl Oscillators{
    pub fn new(sample_rate: f32) -> Self {
        let oscillator1 =  Box::new(Sine::new(sample_rate));
        let oscillator2=  Box::new(Sine::new(sample_rate));
        let oscillator3= Box::new(Sine::new(sample_rate));

        Self{
            sample_rate: sample_rate,
            oscillator1,
            oscillator2,
            oscillator3,
            is_unison: false,
            unison_frequency_offset: 0.0,
        }
    }

    pub fn set_oscillator1_type(&mut self, wave_shape: WaveShape) {
        self.oscillator1 = self.get_oscillator_for_wave_shape(wave_shape);
    }

    pub fn set_oscillator2_type(&mut self, wave_shape: WaveShape) {
        self.oscillator2 = self.get_oscillator_for_wave_shape(wave_shape);
    }

    pub fn set_oscillator3_type(&mut self, wave_shape: WaveShape) {
        self.oscillator3 = self.get_oscillator_for_wave_shape(wave_shape);
    }

    pub fn get_oscillator1_next_sample(&mut self, note_frequency: f32, relative_level: f32, modulation: Option<f32>) -> f32 {
        self.oscillator1.next_sample(note_frequency - (note_frequency * self.unison_frequency_offset), modulation) * relative_level
    }

    pub fn get_oscillator2_next_sample(&mut self, note_frequency: f32,  relative_level: f32, modulation: Option<f32>) -> f32 {
        self.oscillator2.next_sample(note_frequency, modulation) * relative_level
    }

    pub fn get_oscillator3_next_sample(&mut self, note_frequency: f32,  relative_level: f32, modulation: Option<f32>) -> f32 {
        self.oscillator3.next_sample(note_frequency + (note_frequency * self.unison_frequency_offset), modulation) * relative_level
    }

    pub fn enable_unison(&mut self, unison_spread_percentage_of_note: f32) {
        self.is_unison = true;
        self.unison_frequency_offset = unison_spread_percentage_of_note/2.0;
    }

    pub fn disable_unison(&mut self) {
        self.is_unison = false;
        self.unison_frequency_offset = 0.0;
    }

    pub fn is_unison(&self) -> bool {
        self.is_unison
    }

    fn get_oscillator_for_wave_shape(&self, wave_shape: WaveShape) -> Box<dyn GenerateSamples + Send + Sync> {

        match wave_shape {
            WaveShape::Noise => Box::new(Noise::new()),
            WaveShape::Ramp => Box::new(Ramp::new(self.sample_rate.clone())),
            WaveShape::Saw => Box::new(Saw::new(self.sample_rate.clone())),
            WaveShape::Sine => Box::new(Sine::new(self.sample_rate.clone())),
            WaveShape::Square => Box::new(Square::new(self.sample_rate.clone())),
            WaveShape::SuperSaw => Box::new(SuperSaw::new(self.sample_rate.clone())),
            WaveShape::Triangle => Box::new(Triangle::new(self.sample_rate.clone())),
        }

    }

}