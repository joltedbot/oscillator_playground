use crate::synth::AmpMode;
use crossbeam_channel::{Receiver, Sender, unbounded};
use slint::SharedString;

#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    UpdateOscillator1Shape(SharedString),
    UpdateOscillator2Shape(SharedString),
    UpdateOscillator3Shape(SharedString),
    UpdateSubOscillatorShape(SharedString),
    UpdateOscillator1Level(f32),
    UpdateOscillator2Level(f32),
    UpdateOscillator3Level(f32),
    UpdateSubOscillatorLevel(f32),
    UpdateOscillatorDetuneActive(bool, f32),
    UpdateOscillatorDetuneValue(f32),
    UpdateOutputLevel(i32),
    UpdateEnvelopeAttack(i32),
    UpdateEnvelopeDecay(i32),
    UpdateEnvelopeRelease(i32),
    UpdateEnvelopeSustain(i32),
    UpdateEnvelopeSustainLevel(i32),
    UpdateAmpModeEnvelopeEnabled(bool),
    UpdateGateDutyCycle(f32),
    UpdateGateNoteLength(i32),
    UpdateFilterCutoffValue(i32),
    UpdateFilterResonanceValue(f32),
    UpdateFilterNumberOfPoles(i32),
    ResyncOscillators,
    Start,
    Stop,
    Exit,
}

pub struct Events {
    synth_sender: Sender<EventType>,
    synth_receiver: Receiver<EventType>,
    user_interface_sender: Sender<EventType>,
    user_interface_receiver: Receiver<EventType>,
}

impl Events {
    pub fn new() -> Self {
        let (synth_sender, synth_receiver) = unbounded();
        let (user_interface_sender, user_interface_receiver) = unbounded();

        Events {
            synth_sender,
            synth_receiver,
            user_interface_sender,
            user_interface_receiver,
        }
    }

    pub fn get_synth_sender(&self) -> Sender<EventType> {
        self.synth_sender.clone()
    }

    pub fn get_synth_receiver(&self) -> Receiver<EventType> {
        self.synth_receiver.clone()
    }

    pub fn get_user_interface_sender(&self) -> Sender<EventType> {
        self.user_interface_sender.clone()
    }

    pub fn get_user_interface_receiver(&self) -> Receiver<EventType> {
        self.user_interface_receiver.clone()
    }
}
