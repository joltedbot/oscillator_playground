use cpal::{default_host, SampleRate};

const DEFAULT_PWM_FACTOR: f32 = 0.0;
const DEFAULT_PWM_INCREMENT: f32 = 0.0001;
const DEFAULT_PWM_LIMIT: f32 = 0.8;
const DEFAULT_TREMOLO_INCREMENT: f32 = 0.1;
const DEFAULT_ENV_ADJUSTMENT_LEVEL: f32 = 0.0;
const MINIMUM_ENV_LEVEL: f32 = -70.0;
const DEFAULT_ENV_INCREMENT: f32 = 0.001;
const DEFAULT_SUSTAIN_COUNT: u32 = 22050;
const SUSTAIN_LEVEL_BELOW_MIN: f32 = 0.0;

pub enum ADSRStage {
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, PartialEq)]
pub enum State {
    Playing(f32),
    Stopped,
}

pub struct ADSR {
    pub max_level: f32,
    pub current_level: f32,
    pub attack_increment: f32,
    pub decay_increment: f32,
    pub release_increment: f32,
    pub sustain_count: u32,
    pub sustain_length: u32,
    pub sustain_level: f32,
    pub stage: ADSRStage,
    pub state: State,
}

pub struct Modulation {
    sample_rate: u32,
    pwm_factor: f32,
    pwm_increment: f32,
    pwm_limit: f32,
    envelope: ADSR,
}

impl Modulation { 

    pub fn new(sample_rate: u32, output_level: f32) -> Self {
        Self {
            sample_rate,
            pwm_factor: DEFAULT_PWM_FACTOR,
            pwm_increment: DEFAULT_PWM_INCREMENT,
            pwm_limit: DEFAULT_PWM_LIMIT,
            envelope: ADSR {
                current_level: MINIMUM_ENV_LEVEL,
                attack_increment: DEFAULT_ENV_INCREMENT,
                decay_increment: DEFAULT_ENV_INCREMENT,
                release_increment: DEFAULT_ENV_INCREMENT,
                sustain_count: 0,
                sustain_length: DEFAULT_SUSTAIN_COUNT,
                sustain_level: output_level,
                max_level: output_level,
                stage: ADSRStage::Attack,
                state: State::Stopped,
            }
        }
        
        
    }
    
    // pwm_amount represents a scaling factor for the modulation or the width of the pwm or the max change in duty cycle.
    pub fn pwm(&mut self, pwm_amount: f32) -> f32 {

        if self.pwm_factor >= self.pwm_limit || self.pwm_factor <= (-1.0 * self.pwm_limit) {
            self.pwm_increment *= -1.0;
        }

        self.pwm_factor += self.pwm_increment;
        self.pwm_factor * pwm_amount
    }

    
    // This is a hack to give full control in main for testing but it should be broking out into 
    // individual functions to allow individually setting things that should be changeable
    pub fn set_adsr(&mut self, adsr: ADSR) {
        self.envelope = adsr;
    }

    pub fn envelope(&mut self, output_level: f32) -> State {
        
        if self.envelope.state == State::Stopped {
            self.envelope.state = State::Playing(MINIMUM_ENV_LEVEL);
        }
        
        match self.envelope.stage {
            ADSRStage::Attack => {
                if self.envelope.current_level < output_level {
                    self.envelope.current_level += self.envelope.attack_increment;
                } else {
                    println!("Attack -> Decay");
                    self.envelope.current_level = output_level;
                    self.envelope.stage = ADSRStage::Decay;
                }
            },
            ADSRStage::Decay => {
                if self.envelope.current_level > self.envelope.sustain_level {
                    self.envelope.current_level -= self.envelope.decay_increment;
                } else {
                    println!("Decay -> Sustain");
                    self.envelope.current_level = self.envelope.sustain_level;
                    self.envelope.stage = ADSRStage::Sustain;
                }
            }
            ADSRStage::Sustain => {
                if self.envelope.sustain_count < self.envelope.sustain_length {
                    self.envelope.sustain_count += 1;
                } else {
                    println!("Sustain -> Release");
                    self.envelope.stage = ADSRStage::Release;
                    self.envelope.sustain_count = 0;
                }
            },
            ADSRStage::Release => {
                if self.envelope.current_level > MINIMUM_ENV_LEVEL {
                    self.envelope.current_level -= self.envelope.release_increment;
                } else {
                    println!("Release Ends");
                    self.envelope.current_level = MINIMUM_ENV_LEVEL;
                    self.envelope.stage = ADSRStage::Attack;
                    self.envelope.state = State::Stopped;
                }
            }
        }
        if self.envelope.state == State::Stopped {
            return State::Stopped;
        }
        
        State::Playing(get_output_level_adjustment_factor(self.envelope.current_level))
    }
    
}

fn get_output_level_adjustment_factor(output_level: f32) -> f32 {
    10.0_f32.powf(output_level / 20.0)
}

pub fn get_number_of_samples_from_milliseconds(sample_rate: u32, milliseconds: u32) -> u32 {
    sample_rate/1000 * milliseconds
}