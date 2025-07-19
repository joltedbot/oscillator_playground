use crate::synth::LFOParameters;
use crate::synth::lfo::LFO;
use std::sync::MutexGuard;

const PHASER_MAX_WIDTH_VALUE: usize = 126;
const WAVE_SHAPER_MAX_AMOUNT: f32 = 0.9;

pub fn get_wave_shaped_sample(sample: f32, mut amount: f32) -> f32 {
    if amount == 0.0 {
        return sample;
    }

    if amount >= WAVE_SHAPER_MAX_AMOUNT {
        amount = WAVE_SHAPER_MAX_AMOUNT;
    }

    let shape = (2.0 * amount) / (1.0 - amount);
    ((1.0 + shape) * sample) / (1.0 + (shape * sample.abs()))
}

pub fn get_phased_sample(
    lfo: &mut LFO,
    phaser: &mut LFOParameters,
    delay_buffer: &mut MutexGuard<Vec<f32>>,
    original_sample: f32,
) -> f32 {
    delay_buffer.insert(0, original_sample);

    let _trash = delay_buffer.pop();

    let phase_shift = lfo.get_next_value(phaser.frequency, phaser.center_value, phaser.width);
    (original_sample + delay_buffer[PHASER_MAX_WIDTH_VALUE - (phase_shift.round() as usize)]) / 2.0
}

pub fn get_phaser_lfo_center_value_from_amount(amount: f32) -> f32 {
    (PHASER_MAX_WIDTH_VALUE as f32 - (amount / 2.0)).floor()
}

pub fn get_bitcrush_sample(original_sample: f32, new_bit_depth: u32) -> f32 {
    let bits = (2_u32.pow(new_bit_depth) / 2) as f32;
    let quantized = (original_sample.abs() * bits).ceil();
    let mut bitcrushed_sample = quantized / bits;

    if original_sample.is_sign_negative() {
        bitcrushed_sample *= -1.0;
    }

    bitcrushed_sample
}

pub fn get_auto_pan_value(
    lfo: &mut LFO,
    auto_pan: &mut LFOParameters,
    mut left_sample: f32,
    mut right_sample: f32,
) -> (f32, f32) {
    let pan_value = lfo.get_next_value(auto_pan.frequency, auto_pan.center_value, auto_pan.width);

    left_sample *= pan_value;
    right_sample *= 2.0 - pan_value;

    (left_sample, right_sample)
}

pub fn get_tremolo_value(
    lfo: &mut LFO,
    tremolo: &mut LFOParameters,
    mut left_sample: f32,
    mut right_sample: f32,
) -> (f32, f32) {
    let tremolo_value = lfo.get_next_value(tremolo.frequency, tremolo.center_value, tremolo.width);
    left_sample *= tremolo_value;
    right_sample *= tremolo_value;

    (left_sample, right_sample)
}
