const WAVE_SHAPER_MAX_AMOUNT: f32 = 0.99;
const WAVE_SHAPER_AMOUNT_CORRECTION_FACTOR: f32 = 10.0;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum SaturationMode {
    #[default]
    AnalogModeled,
    TubeLike,
    SoftClipping,
    WaveShaping,
    SineShaper,
    Polynomial,
}

pub fn get_saturation_mode_from_mode_name (mode_name: &str) -> SaturationMode {
    match mode_name {
        "Analog Modeled" =>  SaturationMode::AnalogModeled,
        "Tube Like" =>  SaturationMode::TubeLike,
        "Soft Clipping" =>  SaturationMode::SoftClipping,
        "Wave Shaping" =>  SaturationMode::WaveShaping,
        "Sine Shaper" =>  SaturationMode::SineShaper,
        "Polynomial" =>  SaturationMode::Polynomial,
        _ => Default::default(),
    }
}

pub fn get_saturated_samples(mode: SaturationMode, mut amount: f32, left_sample: f32, right_sample: f32) -> (f32, f32) {
    if amount == 0.0 {
        return (left_sample, right_sample);
    }

    if amount > WAVE_SHAPER_MAX_AMOUNT {
        amount = WAVE_SHAPER_MAX_AMOUNT;
    }

    match mode {
        SaturationMode::AnalogModeled => {
            let left_saturation_sample = saturation_analog_modeled(left_sample, amount);
            let right_saturation_sample = saturation_analog_modeled(right_sample, amount);
            (left_saturation_sample, right_saturation_sample)
        },
        SaturationMode::TubeLike => {
            let left_saturation_sample = saturation_exponential_tube_like(left_sample, amount);
            let right_saturation_sample = saturation_exponential_tube_like(right_sample, amount);
            (left_saturation_sample, right_saturation_sample)
        },
        SaturationMode::SoftClipping => {
            let left_saturation_sample = saturation_cubic_soft_clipping(left_sample, amount);
            let right_saturation_sample = saturation_cubic_soft_clipping(right_sample, amount);
            (left_saturation_sample, right_saturation_sample)
        },
        SaturationMode::WaveShaping => {
            let left_saturation_sample = saturation_asymptotic_waveshaper(left_sample, amount);
            let right_saturation_sample = saturation_asymptotic_waveshaper(right_sample, amount);
            (left_saturation_sample, right_saturation_sample)
        },
        SaturationMode::SineShaper => {
            let left_saturation_sample = saturation_sine_shaper(left_sample, amount);
            let right_saturation_sample = saturation_sine_shaper(right_sample, amount);
            (left_saturation_sample, right_saturation_sample)
        },
        SaturationMode::Polynomial => {
            let left_saturation_sample = saturation_chebyshev_polynomial(left_sample, amount);
            let right_saturation_sample = saturation_chebyshev_polynomial(right_sample, amount);
            (left_saturation_sample, right_saturation_sample)
        },
    }
}


fn saturation_analog_modeled(sample: f32, amount: f32) -> f32 {
    let drive = 1.0 + amount * 9.0;
    let shaped = (sample * drive).atan() * (2.0 / std::f32::consts::PI);
    let makeup = 1.0 + (1.0 - amount).powf(0.5) * amount * 3.0;
    shaped * makeup
}

fn saturation_exponential_tube_like(sample: f32, amount: f32) -> f32 {
    let factor = amount * 2.0;
    let shaped = sample.signum() * (1.0 - (-sample.abs() * factor).exp());
    let makeup = 1.0 + amount * (3.0 - amount * 1.5);
    shaped * makeup
}

fn saturation_cubic_soft_clipping(sample: f32, amount: f32) -> f32 {
    let drive = amount * 3.0;
    let x = sample * drive;
    let shaped = if x.abs() < 1.0 {
        x - (x.powi(3) / 3.0)
    } else {
        x.signum() * (2.0 / 3.0)
    };

    let makeup = 1.0 + amount * (2.0 - amount);
    shaped * makeup
}

fn saturation_asymptotic_waveshaper(sample: f32, amount: f32) -> f32 {
    let shape = (2.0 * amount) / (1.0 - amount);
    let shaped = ((1.0 + shape) * sample) / (1.0 + (shape * sample.abs()));
    let makeup = 1.0 + amount * 0.2;
    shaped * makeup
}


fn saturation_sine_shaper(sample: f32, amount: f32) -> f32 {
    let drive = amount * std::f32::consts::PI * 0.5;
    let shaped = (sample * drive).sin();
    let makeup = 1.0 + amount * (1.5 - amount * 0.5);
    shaped * makeup
}

fn saturation_chebyshev_polynomial(sample: f32, amount: f32) -> f32 {
    let x = sample.clamp(-1.0, 1.0);
    let t3 = 4.0 * x.powi(3) - 3.0 * x;
    let t3_scale = 0.25 + amount * 0.5;
    x * (1.0 - amount) + t3 * amount * t3_scale
}

