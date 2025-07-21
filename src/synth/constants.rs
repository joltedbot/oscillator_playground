// Level/Volume Constants
pub const OUTPUT_LEVEL: f32 = -10.0;
pub const UNBLANCED_OUTPUT_LEVEL_ADJUSTMENT: f32 = 3.0;

// Oscillator Constants
pub const DEFAULT_OSC_INTERVAL: i8 = 0;

// Arpeggiator Constants
pub const ARPEGGIATOR_DEFAULT_RANDOMIZE_STATE: bool = false;
pub const DEFAULT_SEQUENCER_NOTE: u16 = 60;

// Effects Constants
pub const DEFAULT_COMPRESSOR_THRESHOLD: f32 = 0.0;
pub const DEFAULT_BITCRUSHER_DEPTH: u32 = 8;
pub const DEFAULT_WAVE_SHAPER_AMOUNT: f32 = 0.0;
pub const DEFAULT_AUTO_PAN_CENTER_VALUE: f32 = 1.0;
pub const DEFAULT_PHASER_CENTER_VALUE: f32 = 87.0;
pub const DEFAULT_PHASER_WIDTH: f32 = 40.0;
pub const DEFAULT_LFO_FREQUENCY: f32 = 1.0;
pub const DEFAULT_OSC_MOD_FREQUENCY: f32 = 0.01;
pub const DEFAULT_OSC_MOD_CENTER_VALUE: f32 = 1.0;
pub const DEFAULT_COMPRESSOR_RATIO: f32 = 0.5;

// LFO Constants
pub const LFO_INDEX_FOR_AUTO_PAN: usize = 0;
pub const LFO_INDEX_FOR_TREMOLO: usize = 1;
pub const LFO_INDEX_FOR_FILTER_MOD: usize = 2;
pub const LFO_INDEX_FOR_SUB_OSCILLATOR_MOD: usize = 3;
pub const LFO_INDEX_FOR_OSCILLATOR1_MOD: usize = 4;
pub const LFO_INDEX_FOR_OSCILLATOR2_MOD: usize = 5;
pub const LFO_INDEX_FOR_OSCILLATOR3_MOD: usize = 6;
pub const OSC_MOD_LFO_INDEX_FOR_SUB: usize = 0;
pub const OSC_MOD_LFO_INDEX_FOR_OSC1: usize = 1;
pub const OSC_MOD_LFO_INDEX_FOR_OSC2: usize = 2;
pub const OSC_MOD_LFO_INDEX_FOR_OSC3: usize = 3;
pub const LFO_INDEX_FOR_PHASE_DELAY: usize = 7;
pub const DEFAULT_CENTER_VALUE: f32 = 0.5;
