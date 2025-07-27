pub mod fm;
pub mod noise;
pub mod pulse;
pub mod ramp;
pub mod saw;
pub mod sine;
pub mod square;
pub mod sub;
pub mod super_saw;
pub mod triangle;
pub mod am;

use fm::FM;
use am::AM;
use noise::Noise;
use pulse::Pulse;
use ramp::Ramp;
use saw::Saw;
use sine::Sine;
use slint::SharedString;
use square::Square;
use sub::Sub;
use super_saw::SuperSaw;
use triangle::Triangle;

const WAVE_SHAPER_MAX_AMOUNT: f32 = 0.9;
const DEFAULT_WAVE_LEVEL: f32 = 1.0;
const DEFAULT_SUB_LEVEL: f32 = 0.0;
const DEFAULT_WAVE_SHAPER_AMOUNT: f32 = 0.0;
const DEFAULT_WAVE_INTERVAL: i32 = 0;

pub trait GenerateSamples {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32;

    fn set_shape_specific_parameter(&mut self, parameter: f32);

    fn reset(&mut self);
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum WaveShape {
    Noise,
    Pulse,
    Ramp,
    Saw,
    #[default]
    Sine,
    Square,
    SuperSaw,
    Triangle,
    FM,
    AM,
}

struct Parameters {
    wave: Box<dyn GenerateSamples + Send + Sync>,
    shape: WaveShape,
    level: f32,
    shaper_amount: f32,
    interval: i32,
}

pub struct Oscillators {
    sample_rate: f32,
    is_unison: bool,
    unison_frequency_offset: f32,
    oscillators: [Parameters; 4],
}

impl Oscillators {
    pub fn new(sample_rate: f32) -> Self {
        let sub_oscillator = Parameters {
            wave: Box::new(Sine::new(sample_rate)),
            shape: WaveShape::Sine,
            level: DEFAULT_SUB_LEVEL,
            shaper_amount: DEFAULT_WAVE_SHAPER_AMOUNT,
            interval: DEFAULT_WAVE_INTERVAL,
        };

        let oscillator1 = Parameters {
            wave: Box::new(Sine::new(sample_rate)),
            shape: WaveShape::Sine,
            level: DEFAULT_WAVE_LEVEL,
            shaper_amount: DEFAULT_WAVE_SHAPER_AMOUNT,
            interval: DEFAULT_WAVE_INTERVAL,
        };

        let oscillator2 = Parameters {
            wave: Box::new(Sine::new(sample_rate)),
            shape: WaveShape::Sine,
            level: DEFAULT_WAVE_LEVEL,
            shaper_amount: DEFAULT_WAVE_SHAPER_AMOUNT,
            interval: DEFAULT_WAVE_INTERVAL,
        };

        let oscillator3 = Parameters {
            wave: Box::new(Sine::new(sample_rate)),
            shape: WaveShape::Sine,
            level: DEFAULT_WAVE_LEVEL,
            shaper_amount: DEFAULT_WAVE_SHAPER_AMOUNT,
            interval: DEFAULT_WAVE_INTERVAL,
        };

        Self {
            sample_rate,
            oscillators: [sub_oscillator, oscillator1, oscillator2, oscillator3],
            is_unison: false,
            unison_frequency_offset: 0.0,
        }
    }

    pub fn set_oscillator_type(&mut self, wave_shape: WaveShape, oscillator_number: i32) {
        let new_oscillator = self.get_oscillator_for_wave_shape(&wave_shape);
        let oscillator = &mut self.oscillators[oscillator_number as usize];

        if oscillator_number == 0 {
            oscillator.wave = Box::new(Sub::new(new_oscillator));
        } else {
            oscillator.wave = new_oscillator;
        }

        oscillator.shape = wave_shape;
    }

    pub fn set_oscillator_level(&mut self, level: f32, oscillator: i32) {
        self.oscillators[oscillator as usize].level = level;
    }

    pub fn set_oscillator_interval(&mut self, interval: i32, oscillator: i32) {
        self.oscillators[oscillator as usize].interval = interval;
    }

    pub fn set_oscillator_fm_amount(&mut self, fm_amount: f32, oscillator: i32) {
        self.oscillators[oscillator as usize]
            .wave
            .set_shape_specific_parameter(fm_amount);
    }

    pub fn set_oscillator_pulse_width(&mut self, width: f32, oscillator: i32) {
        self.oscillators[oscillator as usize]
            .wave
            .set_shape_specific_parameter(width);
    }

    pub fn set_oscillator_shaper_amount(&mut self, amount: f32, oscillator: i32) {
        self.oscillators[oscillator as usize].shaper_amount = amount;
    }

    pub fn get_oscillator_interval(&mut self, oscillator: i32) -> i32 {
        self.oscillators[oscillator as usize].interval
    }

    pub fn get_oscillator1_level(&mut self) -> f32 {
        self.oscillators[1].level
    }

    pub fn get_oscillator2_level(&mut self) -> f32 {
        self.oscillators[2].level
    }

    pub fn get_oscillator3_level(&mut self) -> f32 {
        self.oscillators[3].level
    }

    pub fn get_sub_oscillator_level(&mut self) -> f32 {
        self.oscillators[0].level
    }

    pub fn reset(&mut self) {
        self.oscillators
            .iter_mut()
            .for_each(|oscillator| oscillator.wave.reset());
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

        let sample = self.oscillators[1].wave.next_sample(frequency, modulation) * relative_level;
        get_wave_shaped_sample(sample, self.oscillators[1].shaper_amount)
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

        let sample = self.oscillators[2]
            .wave
            .next_sample(note_frequency, modulation)
            * relative_level;
        get_wave_shaped_sample(sample, self.oscillators[2].shaper_amount)
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

        let sample = self.oscillators[3].wave.next_sample(frequency, modulation) * relative_level;
        get_wave_shaped_sample(sample, self.oscillators[3].shaper_amount)
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

        let sample = self.oscillators[0]
            .wave
            .next_sample(note_frequency, modulation)
            * relative_level;
        get_wave_shaped_sample(sample, self.oscillators[0].shaper_amount)
    }

    pub fn enable_unison(&mut self, unison_spread_percentage_of_note: f32) {
        self.is_unison = true;
        self.unison_frequency_offset = unison_spread_percentage_of_note / 2.0;
    }

    pub fn disable_unison(&mut self) {
        self.is_unison = false;
        self.unison_frequency_offset = 0.0;
    }

    pub fn get_oscillator_for_wave_shape(
        &mut self,
        wave_shape: &WaveShape,
    ) -> Box<dyn GenerateSamples + Send + Sync> {
        match wave_shape {
            WaveShape::Noise => Box::new(Noise::new()),
            WaveShape::Pulse => Box::new(Pulse::new(self.sample_rate)),
            WaveShape::Ramp => Box::new(Ramp::new(self.sample_rate)),
            WaveShape::Saw => Box::new(Saw::new(self.sample_rate)),
            WaveShape::Sine => Box::new(Sine::new(self.sample_rate)),
            WaveShape::Square => Box::new(Square::new(self.sample_rate)),
            WaveShape::SuperSaw => Box::new(SuperSaw::new(self.sample_rate)),
            WaveShape::Triangle => Box::new(Triangle::new(self.sample_rate)),
            WaveShape::FM => Box::new(FM::new(self.sample_rate)),
            WaveShape::AM => Box::new(AM::new(self.sample_rate)),
        }
    }

    pub fn get_wave_shape_from_shape_name(&self, wave_shape: SharedString) -> WaveShape {
        match wave_shape.as_str() {
            "Noise" => WaveShape::Noise,
            "Pulse" => WaveShape::Pulse,
            "Ramp" => WaveShape::Ramp,
            "Saw" => WaveShape::Saw,
            "Sine" => WaveShape::Sine,
            "Square" => WaveShape::Square,
            "SuperSaw" => WaveShape::SuperSaw,
            "Triangle" => WaveShape::Triangle,
            "FM" => WaveShape::FM,
            "AM" => WaveShape::AM,
            _ => WaveShape::Sine,
        }
    }
}

fn get_wave_shaped_sample(sample: f32, mut amount: f32) -> f32 {
    if amount == 0.0 {
        return sample;
    }

    if amount >= WAVE_SHAPER_MAX_AMOUNT {
        amount = WAVE_SHAPER_MAX_AMOUNT;
    }

    let shape = (2.0 * amount) / (1.0 - amount);
    ((1.0 + shape) * sample) / (1.0 + (shape * sample.abs()))
}
