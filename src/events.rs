use crate::device_manager::DeviceList;
use crossbeam_channel::{Receiver, Sender, unbounded};
use slint::SharedString;

#[derive(Clone, PartialEq, Debug)]
pub enum EventType {
    UpdateOscillatorShape(SharedString, i32),
    UpdateOscillatorTuning(i32, i32),
    UpdateOscillatorLevel(f32, i32),
    UpdateOscillatorSpecificParameters((f32, f32), i32),
    UpdateOscillatorShaperAmount(f32, i32),
    UpdateOscillatorModFreq(f32, i32),
    UpdateOscillatorModAmount(f32, i32),
    UpdateOscillatorDetuneActive(bool, f32),
    UpdateOscillatorDetuneValue(f32),
    UpdateOutputLevel(i32),
    UpdateOutputLevelConstant(bool),
    UpdateEnvelopeAttack(i32),
    UpdateEnvelopeDecay(i32),
    UpdateEnvelopeRelease(i32),
    UpdateEnvelopeSustainLevel(i32),
    UpdateADSRNoteLength(i32),
    UpdateAmpModeEnvelopeEnabled(bool),
    UpdateGateDutyCycle(f32),
    UpdateGateNoteLength(i32),
    UpdateFilterCutoffValue(i32),
    UpdateFilterResonanceValue(f32),
    UpdateFilterNumberOfPoles(i32),
    UpdateAutoPanEnabled(bool),
    UpdateAutoPanSpeed(f32),
    UpdateAutoPanWidth(f32),
    UpdateTremoloEnabled(bool),
    UpdateTremoloSpeed(f32),
    UpdateTremoloDepth(f32),
    UpdateFilterModEnabled(bool),
    UpdateFilterModSpeed(f32),
    UpdateFilterModAmount(f32),
    UpdateFilterModShape(SharedString),
    UpdatePhaserEnabled(bool),
    UpdatePhaserSpeed(f32),
    UpdatePhaserAmount(f32),
    UpdateBitCrusherEnabled(bool),
    UpdateBitCrusherAmount(i32),
    UpdateWaveShaperEnabled(bool),
    UpdateWaveShaperAmount(f32),
    UpdateCompressorActive(bool),
    UpdateCompressorThreshold(f32),
    UpdateCompressorRatio(f32),
    UpdateWavefolderActive(bool),
    UpdateWavefolderThreshold(f32),
    UpdateWavefolderRatio(f32),
    UpdateLimiterActive(bool),
    UpdateLimiterThreshold(f32),
    UpdateClipperActive(bool),
    UpdateClipperThreshold(f32),
    ResyncOscillators,
    ResyncOscillatorLFOs,
    ArpeggiatorActive(bool),
    ArpeggiatorAddNote(i32),
    ArpeggiatorRemoveNote(i32),
    ArpeggiatorRandomEnabled(bool),
    MidiNoteOn(u8),
    MidiNoteOff(u8),
    UpdateMidiPortList(Vec<String>),
    UpdateMidiPort(i32),
    UpdateMidiChannel(i32),
    UpdateOutputDeviceList(DeviceList),
    UpdateAudioDevice(String),
    UpdateAudioChannels(String, String),
}

pub struct Events {
    synth_sender: Sender<EventType>,
    synth_receiver: Receiver<EventType>,
    ui_sender: Sender<EventType>,
    ui_receiver: Receiver<EventType>,
    midi_sender: Sender<EventType>,
    midi_receiver: Receiver<EventType>,
}

impl Events {
    pub fn new() -> Self {
        let (synth_sender, synth_receiver) = unbounded();
        let (ui_sender, ui_receiver) = unbounded();
        let (midi_sender, midi_receiver) = unbounded();

        Events {
            synth_sender,
            synth_receiver,
            ui_sender,
            ui_receiver,
            midi_sender,
            midi_receiver,
        }
    }

    pub fn get_synth_sender(&self) -> Sender<EventType> {
        self.synth_sender.clone()
    }

    pub fn get_synth_receiver(&self) -> Receiver<EventType> {
        self.synth_receiver.clone()
    }

    pub fn get_ui_sender(&self) -> Sender<EventType> {
        self.ui_sender.clone()
    }

    pub fn get_ui_receiver(&self) -> Receiver<EventType> {
        self.ui_receiver.clone()
    }

    pub fn get_midi_sender(&self) -> Sender<EventType> {
        self.midi_sender.clone()
    }

    pub fn get_midi_receiver(&self) -> Receiver<EventType> {
        self.midi_receiver.clone()
    }
}
