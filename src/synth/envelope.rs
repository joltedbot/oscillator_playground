const MINIMUM_ENV_LEVEL: f32 = -70.0;
const DEFAULT_ATTACK_MILLISECONDS: u32 = 50;
const DEFAULT_DECAY_MILLISECONDS: u32 = 300;
const DEFAULT_RELEASE_MILLISECONDS: u32 = 200;
const DEFAULT_SUSTAIN_MILLISECONDS: u32 = 300;
const DEFAULT_SUSTAIN_LEVEL_BELOW_OUTPUT_LEVEL: f32 = 0.0;

#[derive(Clone, PartialEq)]
pub enum ADSRStage {
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, PartialEq)]
pub enum EnvelopeState {
    Playing(f32),
    Stopped,
}

#[derive(Clone, PartialEq)]
struct ADSR {
    current_level: f32,
    attack_milliseconds: u32,
    decay_milliseconds: u32,
    release_milliseconds: u32,
    sustain_count: u32,
    sustain_length: u32,
    sustain_level: f32,
    stage: ADSRStage,
    state: EnvelopeState,
}

#[derive(Clone, PartialEq)]
pub struct Envelope {
    sample_rate: u32,
    envelope: ADSR,
}

impl Envelope {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            envelope: ADSR {
                current_level: MINIMUM_ENV_LEVEL,
                attack_milliseconds: DEFAULT_ATTACK_MILLISECONDS,
                decay_milliseconds: DEFAULT_DECAY_MILLISECONDS,
                release_milliseconds: DEFAULT_RELEASE_MILLISECONDS,
                sustain_count: 0,
                sustain_length: DEFAULT_SUSTAIN_MILLISECONDS,
                sustain_level: DEFAULT_SUSTAIN_LEVEL_BELOW_OUTPUT_LEVEL,
                stage: ADSRStage::Attack,
                state: EnvelopeState::Stopped,
            },
        }
    }

    pub fn set_attack_milliseconds(&mut self, milliseconds: u32) {
        self.envelope.attack_milliseconds = milliseconds;
    }

    pub fn set_decay_milliseconds(&mut self, milliseconds: u32) {
        self.envelope.decay_milliseconds = milliseconds;
    }

    pub fn set_release_milliseconds(&mut self, milliseconds: u32) {
        self.envelope.release_milliseconds = milliseconds;
    }

    pub fn set_sustain_length_milliseconds(&mut self, millliseconds: u32) {
        self.envelope.sustain_length = millliseconds;
    }

    pub fn set_sustain_level_below_output_level_in_dbfs(&mut self, level: f32) {
        self.envelope.sustain_level = level;
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }

    pub fn adsr(&mut self, output_level: f32) -> EnvelopeState {
        if self.envelope.state == EnvelopeState::Stopped {
            self.envelope.state = EnvelopeState::Playing(MINIMUM_ENV_LEVEL);
        }

        match self.envelope.stage {
            ADSRStage::Attack => {
                if self.envelope.current_level < output_level {
                    self.envelope.current_level += self.get_increment_from_milliseconds(
                        self.envelope.attack_milliseconds,
                        MINIMUM_ENV_LEVEL,
                        output_level,
                    );
                } else {
                    self.envelope.current_level = output_level;
                    self.envelope.stage = ADSRStage::Decay;
                }
            }
            ADSRStage::Decay => {
                if self.envelope.current_level > (output_level - self.envelope.sustain_level) {
                    self.envelope.current_level -= self.get_increment_from_milliseconds(
                        self.envelope.decay_milliseconds,
                        output_level,
                        output_level - self.envelope.sustain_level,
                    );
                } else {
                    self.envelope.current_level = output_level - self.envelope.sustain_level;
                    self.envelope.stage = ADSRStage::Sustain;
                }
            }
            ADSRStage::Sustain => {
                if self.envelope.sustain_count
                    < self.get_number_of_samples_from_milliseconds(self.envelope.sustain_length)
                {
                    self.envelope.sustain_count += 1;
                } else {
                    self.envelope.stage = ADSRStage::Release;
                    self.envelope.sustain_count = 0;
                }
            }
            ADSRStage::Release => {
                if self.envelope.current_level > MINIMUM_ENV_LEVEL {
                    self.envelope.current_level -= self.get_increment_from_milliseconds(
                        self.envelope.release_milliseconds,
                        output_level - self.envelope.sustain_level,
                        MINIMUM_ENV_LEVEL,
                    );
                } else {
                    self.envelope.current_level = MINIMUM_ENV_LEVEL;
                    self.envelope.stage = ADSRStage::Attack;
                    self.envelope.state = EnvelopeState::Stopped;
                }
            }
        }
        if self.envelope.state == EnvelopeState::Stopped {
            return EnvelopeState::Stopped;
        }

        EnvelopeState::Playing(get_output_level_adjustment_factor(
            self.envelope.current_level,
        ))
    }

    fn get_number_of_samples_from_milliseconds(&self, milliseconds: u32) -> u32 {
        self.sample_rate / 1000 * milliseconds
    }

    fn get_increment_from_milliseconds(
        &self,
        milliseconds: u32,
        start_level: f32,
        target_level: f32,
    ) -> f32 {
        let level_delta = start_level - target_level;
        let number_of_samples = self.get_number_of_samples_from_milliseconds(milliseconds);
        level_delta.abs() / (number_of_samples as f32)
    }
}

fn get_output_level_adjustment_factor(output_level: f32) -> f32 {
    10.0_f32.powf(output_level / 20.0)
}
