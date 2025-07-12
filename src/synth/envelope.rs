const MINIMUM_ENV_LEVEL: f32 = -60.0;
const DEFAULT_ATTACK_MILLISECONDS: u32 = 50;
const DEFAULT_DECAY_MILLISECONDS: u32 = 300;
const DEFAULT_RELEASE_MILLISECONDS: u32 = 200;
const DEFAULT_SUSTAIN_MILLISECONDS: u32 = 300;
const DEFAULT_SUSTAIN_LEVEL_BELOW_OUTPUT_LEVEL: f32 = 0.0;
const DEFAULT_GATE_DUTY_CYCLE: f32 = 0.5;
const GATE_OFF_SAMPLE_VALUE: f32 = 0.0;
const MAXIMUM_GATE_DUTY_CYCLE: f32 = 0.9;
const DEFAULT_GATE_NOTE_LENGTH_MILLISECONDS: f32 = 800.0;
const DEFAULT_STATE_COUNT_VALUE: u32 = 0;

#[derive(Clone, PartialEq)]
pub enum ADSRStage {
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, PartialEq)]
pub enum GateState {
    On(f32),
    Off(f32),
    End(f32),
}

#[derive(Clone, PartialEq)]
pub enum ADSRState {
    Playing(f32),
    Stopped,
}

#[derive(Clone, PartialEq)]
struct Gate {
    duty_cycle: f32,
    on_sample_count: u32,
    on_maximum_samples: u32,
    off_sample_count: u32,
    off_maximum_samples: u32,
    note_length_milliseconds: f32,
    state: GateState,
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
    state: ADSRState,
}

#[derive(Clone, PartialEq)]
pub struct Envelope {
    sample_rate: u32,
    envelope: ADSR,
    gate: Gate,
}

impl Envelope {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            gate: Gate {
                duty_cycle: DEFAULT_GATE_DUTY_CYCLE,
                on_sample_count: DEFAULT_STATE_COUNT_VALUE,
                on_maximum_samples: get_number_of_samples_from_milliseconds(
                    sample_rate,
                    (DEFAULT_GATE_NOTE_LENGTH_MILLISECONDS * DEFAULT_GATE_DUTY_CYCLE).round()
                        as u32,
                ),
                off_sample_count: DEFAULT_STATE_COUNT_VALUE,
                off_maximum_samples: get_number_of_samples_from_milliseconds(
                    sample_rate,
                    (DEFAULT_GATE_NOTE_LENGTH_MILLISECONDS * (1.0 - DEFAULT_GATE_DUTY_CYCLE))
                        .round() as u32,
                ),
                note_length_milliseconds: DEFAULT_GATE_NOTE_LENGTH_MILLISECONDS,
                state: GateState::On(GATE_OFF_SAMPLE_VALUE),
            },
            envelope: ADSR {
                current_level: MINIMUM_ENV_LEVEL,
                attack_milliseconds: DEFAULT_ATTACK_MILLISECONDS,
                decay_milliseconds: DEFAULT_DECAY_MILLISECONDS,
                release_milliseconds: DEFAULT_RELEASE_MILLISECONDS,
                sustain_count: DEFAULT_STATE_COUNT_VALUE,
                sustain_length: DEFAULT_SUSTAIN_MILLISECONDS,
                sustain_level: DEFAULT_SUSTAIN_LEVEL_BELOW_OUTPUT_LEVEL,
                stage: ADSRStage::Attack,
                state: ADSRState::Stopped,
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

    pub fn set_sustain_milliseconds(&mut self, millliseconds: u32) {
        self.envelope.sustain_length = millliseconds;
    }

    pub fn set_sustain_level_below_output_level_in_dbfs(&mut self, level: f32) {
        self.envelope.sustain_level = level;
    }

    pub fn set_gate_duty_cycle(&mut self, duty_cycle: f32) {
        if duty_cycle <= 0.0 {
            self.gate.duty_cycle = 0.0;
        }

        if duty_cycle >= MAXIMUM_GATE_DUTY_CYCLE {
            self.gate.duty_cycle = MAXIMUM_GATE_DUTY_CYCLE;
        }

        self.gate.duty_cycle = duty_cycle;

        let total_number_of_samples = get_number_of_samples_from_milliseconds(
            self.sample_rate,
            self.gate.note_length_milliseconds.round() as u32,
        ) as f32;
        self.gate.on_maximum_samples =
            (total_number_of_samples * self.gate.duty_cycle).round() as u32;
        self.gate.off_maximum_samples =
            (total_number_of_samples * (1.0 - self.gate.duty_cycle)).round() as u32;
    }

    pub fn set_gate_note_length(&mut self, note_length_milliseconds: u32) {
        let total_number_of_samples =
            get_number_of_samples_from_milliseconds(self.sample_rate, note_length_milliseconds)
                as f32;
        self.gate.on_maximum_samples =
            (total_number_of_samples * self.gate.duty_cycle).round() as u32;
        self.gate.off_maximum_samples =
            (total_number_of_samples * (1.0 - self.gate.duty_cycle)).round() as u32;
        self.gate.note_length_milliseconds = note_length_milliseconds as f32;
    }

    pub fn gate(&mut self, output_level: f32) -> GateState {
        match self.gate.state {
            GateState::On(_) => {
                if self.gate.on_sample_count >= self.gate.on_maximum_samples {
                    self.gate.state = GateState::Off(GATE_OFF_SAMPLE_VALUE);
                    self.gate.on_sample_count = DEFAULT_STATE_COUNT_VALUE;
                    return self.gate.state.clone();
                }

                self.gate.on_sample_count += 1;
                GateState::On(get_f32_sample_from_dbfs(output_level))
            }
            GateState::Off(_) => {
                if self.gate.off_sample_count >= self.gate.off_maximum_samples {
                    self.gate.state = GateState::End(GATE_OFF_SAMPLE_VALUE);
                    self.gate.off_sample_count = DEFAULT_STATE_COUNT_VALUE;
                    return GateState::Off(GATE_OFF_SAMPLE_VALUE);
                }

                self.gate.off_sample_count += 1;
                GateState::Off(GATE_OFF_SAMPLE_VALUE)
            }
            GateState::End(_) => {
                self.gate.state = GateState::On(output_level);
                GateState::End(GATE_OFF_SAMPLE_VALUE)
            }
        }
    }

    pub fn adsr(&mut self, output_level: f32) -> ADSRState {
        if self.envelope.state == ADSRState::Stopped {
            self.envelope.state = ADSRState::Playing(MINIMUM_ENV_LEVEL);
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
                    < get_number_of_samples_from_milliseconds(
                        self.sample_rate,
                        self.envelope.sustain_length,
                    )
                {
                    self.envelope.sustain_count += 1;
                } else {
                    self.envelope.stage = ADSRStage::Release;
                    self.envelope.sustain_count = DEFAULT_STATE_COUNT_VALUE;
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
                    self.envelope.state = ADSRState::Stopped;
                }
            }
        }
        if self.envelope.state == ADSRState::Stopped {
            return ADSRState::Stopped;
        }

        ADSRState::Playing(get_f32_sample_from_dbfs(self.envelope.current_level))
    }

    fn get_increment_from_milliseconds(
        &self,
        milliseconds: u32,
        start_level: f32,
        target_level: f32,
    ) -> f32 {
        let level_delta = start_level - target_level;
        let number_of_samples =
            get_number_of_samples_from_milliseconds(self.sample_rate, milliseconds);
        level_delta.abs() / (number_of_samples as f32)
    }
}

fn get_number_of_samples_from_milliseconds(sample_rate: u32, milliseconds: u32) -> u32 {
    ((sample_rate as f64 / 1000.0) * milliseconds as f64).round() as u32
}

fn get_f32_sample_from_dbfs(output_level: f32) -> f32 {
    10.0_f32.powf(output_level / 20.0)
}
