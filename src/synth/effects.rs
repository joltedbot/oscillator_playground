use crate::synth::LFOParameters;
use crate::synth::lfo::LFO;
use std::sync::MutexGuard;

const PHASER_MAX_WIDTH_VALUE: usize = 126;
const WAVE_SHAPER_MAX_AMOUNT: f32 = 0.9;

pub fn get_wave_shaped_sample(mut amount: f32, left_sample: f32, right_sample: f32) -> (f32, f32) {
    if amount == 0.0 {
        return (left_sample, right_sample);
    }

    if amount >= WAVE_SHAPER_MAX_AMOUNT {
        amount = WAVE_SHAPER_MAX_AMOUNT;
    }

    let shape = (2.0 * amount) / (1.0 - amount);

    let left_shaped_sample = ((1.0 + shape) * left_sample) / (1.0 + (shape * left_sample.abs()));
    let right_shaped_sample = ((1.0 + shape) * right_sample) / (1.0 + (shape * right_sample.abs()));

    (left_shaped_sample, right_shaped_sample)
}

pub fn get_phased_sample(
    lfo: &mut LFO,
    phaser: &mut LFOParameters,
    delay_buffer: &mut MutexGuard<Vec<(f32, f32)>>,
    left_sample: f32,
    right_sample: f32,
) -> (f32, f32) {
    delay_buffer.insert(0, (left_sample, right_sample));

    let _trash = delay_buffer.pop();


    let phase_shift = lfo.get_next_value(phaser.frequency, phaser.center_value, phaser.width);
    let left_phased_sample = (left_sample + delay_buffer[PHASER_MAX_WIDTH_VALUE - (phase_shift.round() as usize)].0) / 2.0;
    let right_phased_sample = (right_sample + delay_buffer[PHASER_MAX_WIDTH_VALUE - (phase_shift.round() as usize)].1) / 2.0;

    (left_phased_sample, right_phased_sample)

}

pub fn get_phaser_lfo_center_value_from_amount(amount: f32) -> f32 {
    (PHASER_MAX_WIDTH_VALUE as f32 - (amount / 2.0)).floor()
}



pub fn get_bitcrush_sample(new_bit_depth: u32, left_sample: f32, right_sample: f32) -> (f32, f32) {

    let bits = (2_u32.pow(new_bit_depth) / 2) as f32;

    let left_quantized_sample = (left_sample.abs() * bits).ceil();
    let mut left_bitcrushed_sample = left_quantized_sample / bits;

    let right_quantized_sample = (right_sample.abs() * bits).ceil();
    let mut right_bitcrushed_sample = right_quantized_sample / bits;

    if left_sample.is_sign_negative() {
        left_bitcrushed_sample *= -1.0;
    }

    if right_sample.is_sign_negative() {
        right_bitcrushed_sample *= -1.0;
    }

    (left_bitcrushed_sample, right_bitcrushed_sample)
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
