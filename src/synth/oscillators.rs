pub mod noise;
pub mod ramp;
pub mod saw;
pub mod sine;
pub mod square;
pub mod sub;
pub mod super_saw;
pub mod triangle;

use noise::Noise;
use ramp::Ramp;
use saw::Saw;
use sine::Sine;
use slint::SharedString;
use square::Square;
use sub::Sub;
use super_saw::SuperSaw;
use triangle::Triangle;

const WAVE_SHAPER_MAX_AMOUNT: f32 = 0.9;

pub trait GenerateSamples {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32;
    fn reset(&mut self);
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
    sub_oscillator: Box<dyn GenerateSamples + Send + Sync>,
    oscillator1_level: f32,
    oscillator2_level: f32,
    oscillator3_level: f32,
    sub_oscillator_level: f32,
    oscillator1_shaper_amount: f32,
    oscillator2_shaper_amount: f32,
    oscillator3_shaper_amount: f32,
    sub_oscillator_shaper_amount: f32,
    is_unison: bool,
    unison_frequency_offset: f32,
}

impl Oscillators {
    pub fn new(sample_rate: f32) -> Self {
        let oscillator1 = Box::new(Sine::new(sample_rate));
        let oscillator2 = Box::new(Sine::new(sample_rate));
        let oscillator3 = Box::new(Sine::new(sample_rate));
        let sub_oscillator = Box::new(Sub::new(Box::new(Sine::new(sample_rate))));
        let oscillator1_level = 1.0;
        let oscillator2_level = 1.0;
        let oscillator3_level = 1.0;
        let sub_oscillator_level = 0.0;
        let oscillator1_shaper_amount = 0.0;
        let oscillator2_shaper_amount = 0.0;
        let oscillator3_shaper_amount = 0.0;
        let sub_oscillator_shaper_amount = 0.0;


        Self {
            sample_rate,
            oscillator1,
            oscillator2,
            oscillator3,
            sub_oscillator,
            oscillator1_level,
            oscillator2_level,
            oscillator3_level,
            sub_oscillator_level,
            oscillator1_shaper_amount,
            oscillator2_shaper_amount,
            oscillator3_shaper_amount,
            sub_oscillator_shaper_amount,
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

    pub fn set_sub_oscillator_type(&mut self, wave_shape: WaveShape) {
        self.sub_oscillator = Box::new(Sub::new(self.get_oscillator_for_wave_shape(wave_shape)));
    }

    pub fn set_oscillator1_level(&mut self, level: f32) {
        self.oscillator1_level = level;
    }

    pub fn set_oscillator2_level(&mut self, level: f32) {
        self.oscillator2_level = level;
    }

    pub fn set_oscillator3_level(&mut self, level: f32) {
        self.oscillator3_level = level;
    }

    pub fn set_sub_oscillator_level(&mut self, level: f32) {
        self.sub_oscillator_level = level;
    }

    pub fn set_oscillator1_shaper_amount(&mut self, amount: f32) {
        self.oscillator1_shaper_amount = amount;
    }

    pub fn set_oscillator2_shaper_amount(&mut self, amount: f32) {
        self.oscillator2_shaper_amount = amount;
    }

    pub fn set_oscillator3_shaper_amount(&mut self, amount: f32) {
        self.oscillator3_shaper_amount = amount;
    }

    pub fn set_sub_oscillator_shaper_amount(&mut self, amount: f32) {
        self.sub_oscillator_shaper_amount = amount;
    }



    pub fn get_oscillator1_level(&mut self) -> f32 {
        self.oscillator1_level
    }

    pub fn get_oscillator2_level(&mut self) -> f32 {
        self.oscillator2_level
    }

    pub fn get_oscillator3_level(&mut self) -> f32 {
        self.oscillator3_level
    }

    pub fn get_sub_oscillator_level(&mut self) -> f32 {
        self.sub_oscillator_level
    }

    pub fn reset(&mut self) {
        self.oscillator1.reset();
        self.oscillator2.reset();
        self.oscillator3.reset();
        self.sub_oscillator.reset();
    }

    pub fn get_oscillator1_next_sample(
        &mut self,
        note_frequency: f32,
        relative_level: f32,
        modulation: Option<f32>,
    ) -> f32 {
        if relative_level == 0.0 || note_frequency == 0.0 {
            return 0.0;
        }

        let frequency = if self.unison_frequency_offset == 0.0 {
            note_frequency
        } else {
            note_frequency - (note_frequency * self.unison_frequency_offset)
        };

        let sample = self.oscillator1.next_sample(frequency, modulation) * relative_level;
        get_wave_shaped_sample(sample, self.oscillator1_shaper_amount)
    }

    pub fn get_oscillator2_next_sample(
        &mut self,
        note_frequency: f32,
        relative_level: f32,
        modulation: Option<f32>,
    ) -> f32 {
        if relative_level == 0.0 || note_frequency == 0.0 {
            return 0.0;
        }

        let sample = self.oscillator2.next_sample(note_frequency, modulation) * relative_level;
        get_wave_shaped_sample(sample, self.oscillator2_shaper_amount)
    }

    pub fn get_oscillator3_next_sample(
        &mut self,
        note_frequency: f32,
        relative_level: f32,
        modulation: Option<f32>,
    ) -> f32 {
        if relative_level == 0.0 || note_frequency == 0.0 {
            return 0.0;
        }

        let frequency = if self.unison_frequency_offset == 0.0 {
            note_frequency
        } else {
            note_frequency - (note_frequency * self.unison_frequency_offset)
        };

        let sample = self.oscillator3.next_sample(frequency, modulation) * relative_level;
        get_wave_shaped_sample(sample, self.oscillator3_shaper_amount)
    }

    pub fn get_sub_oscillator_next_sample(
        &mut self,
        note_frequency: f32,
        relative_level: f32,
        modulation: Option<f32>,
    ) -> f32 {
        if relative_level == 0.0 || note_frequency == 0.0 {
            return 0.0;
        }

        let sample = self.sub_oscillator.next_sample(note_frequency, modulation) * relative_level;
        get_wave_shaped_sample(sample, self.sub_oscillator_shaper_amount)
    }

    pub fn enable_unison(&mut self, unison_spread_percentage_of_note: f32) {
        self.is_unison = true;
        self.unison_frequency_offset = unison_spread_percentage_of_note / 2.0;
    }

    pub fn disable_unison(&mut self) {
        self.is_unison = false;
        self.unison_frequency_offset = 0.0;
    }

    fn get_oscillator_for_wave_shape(
        &self,
        wave_shape: WaveShape,
    ) -> Box<dyn GenerateSamples + Send + Sync> {
        match wave_shape {
            WaveShape::Noise => Box::new(Noise::new()),
            WaveShape::Ramp => Box::new(Ramp::new(self.sample_rate)),
            WaveShape::Saw => Box::new(Saw::new(self.sample_rate)),
            WaveShape::Sine => Box::new(Sine::new(self.sample_rate)),
            WaveShape::Square => Box::new(Square::new(self.sample_rate)),
            WaveShape::SuperSaw => Box::new(SuperSaw::new(self.sample_rate)),
            WaveShape::Triangle => Box::new(Triangle::new(self.sample_rate)),
        }
    }

    pub fn get_wave_shape_from_shape_name(&self, wave_shape: SharedString) -> WaveShape {
        match wave_shape.as_str() {
            "Noise" => WaveShape::Noise,
            "Ramp" => WaveShape::Ramp,
            "Saw" => WaveShape::Saw,
            "Sine" => WaveShape::Sine,
            "Square" => WaveShape::Square,
            "SuperSaw" => WaveShape::SuperSaw,
            "Triangle" => WaveShape::Triangle,
            _ => WaveShape::Sine,
        }
    }
}



fn get_wave_shaped_sample(sample: f32, mut amount: f32) -> f32 {

    if amount == 0.0 {
        return sample
    }

    if amount >= WAVE_SHAPER_MAX_AMOUNT {
        amount = WAVE_SHAPER_MAX_AMOUNT;
    }

    let shape = (2.0 * amount) / (1.0 - amount);
    ((1.0+ shape) * sample) / (1.0 + (shape * sample.abs()))

}
