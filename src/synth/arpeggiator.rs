use rand::prelude::*;

const REST_FREQUENCY: f32 = 0.0;
const NUMBER_OF_MIDI_NOTES: usize = 128;
pub const FIRST_REST_NOTE: u16 = 128;
const MIDI_NOTE_FREQUENCIES: [(&str, f32, u16); NUMBER_OF_MIDI_NOTES] = [
    ("C-1", 8.175, 0),
    ("C#-1/Db-1", 8.662, 1),
    ("D-1", 9.177, 2),
    ("D#-1/Eb-1", 9.722, 3),
    ("E-1", 10.300, 4),
    ("F-1", 10.913, 5),
    ("F#-1/Gb-1", 11.562, 6),
    ("G-1", 12.249, 7),
    ("G#-1/Ab-1", 12.978, 8),
    ("A-1", 13.750, 9),
    ("A#-1/Bb-1", 14.567, 10),
    ("B-1", 15.433, 11),
    ("C0", 16.351, 12),
    ("C#0/Db0", 17.323, 13),
    ("D0", 18.354, 14),
    ("D#0/Eb0", 19.445, 15),
    ("E0", 20.601, 16),
    ("F0", 21.826, 17),
    ("F#0/Gb0", 23.124, 18),
    ("G0", 24.499, 19),
    ("G#0/Ab0", 25.956, 20),
    ("A0", 27.500, 21),
    ("A#0/Bb0", 29.135, 22),
    ("B0", 30.867, 23),
    ("C1", 32.703, 24),
    ("C#1/Db1", 34.647, 25),
    ("D1", 36.708, 26),
    ("D#1/Eb1", 38.890, 27),
    ("E1", 41.203, 28),
    ("F1", 43.653, 29),
    ("F#1/Gb1", 46.249, 30),
    ("G1", 48.999, 31),
    ("G#1/Ab1", 51.913, 32),
    ("A1", 55.000, 33),
    ("A#1/Bb1", 58.270, 34),
    ("B1", 61.735, 35),
    ("C2", 65.406, 36),
    ("C#2/Db2", 69.295, 37),
    ("D2", 73.416, 38),
    ("D#2/Eb2", 77.781, 39),
    ("E2", 82.406, 40),
    ("F2", 87.307, 41),
    ("F#2/Gb2", 92.498, 42),
    ("G2", 97.998, 43),
    ("G#2/Ab2", 103.826, 44),
    ("A2", 110.000, 45),
    ("A#2/Bb2", 116.540, 46),
    ("B2", 123.470, 47),
    ("C3", 130.812, 48), // 48
    ("C#3/Db3", 138.591, 49),
    ("D3", 146.832, 50),
    ("D#3/Eb3", 155.563, 51),
    ("E3", 164.813, 52),
    ("F3", 174.614, 53),
    ("F#3/Gb3", 184.997, 54),
    ("G3", 195.997, 55),
    ("G#3/Ab3", 207.652, 56),
    ("A3", 220.000, 57),
    ("A#3/Bb3", 233.081, 58),
    ("B3", 246.941, 59),
    ("C4 (middle C)", 261.625, 60), // 60
    ("C#4/Db4", 277.182, 61),
    ("D4", 293.664, 62),
    ("D#4/Eb4", 311.127, 63),
    ("E4", 329.627, 64),
    ("F4", 349.228, 65),
    ("F#4/Gb4", 369.994, 66),
    ("G4", 391.995, 67),
    ("G#4/Ab4", 415.304, 68),
    ("A4", 440.000, 69),
    ("A#4/Bb4", 466.163, 70),
    ("B4", 493.883, 71),
    ("C5", 523.251, 72), // 72
    ("C#5/Db5", 554.365, 73),
    ("D5", 587.329, 74),
    ("D#5/Eb5", 622.254, 75),
    ("E5", 659.255, 76),
    ("F5", 698.456, 77),
    ("F#5/Gb5", 739.988, 78),
    ("G5", 783.990, 79),
    ("G#5/Ab5", 830.609, 80),
    ("A5", 880.000, 81),
    ("A#5/Bb5", 932.327, 82),
    ("B5", 987.766, 83),
    ("C6", 1046.502, 84), // 84
    ("C#6/Db6", 1_108.73, 85),
    ("D6", 1174.659, 86),
    ("D#6/Eb6", 1244.507, 87),
    ("E6", 1_318.51, 88),
    ("F6", 1396.912, 89),
    ("F#6/Gb6", 1479.977, 90),
    ("G6", 1567.981, 91),
    ("G#6/Ab6", 1661.218, 92),
    ("A6", 1760.000, 93),
    ("A#6/Bb6", 1864.655, 94),
    ("B6", 1975.533, 95),
    ("C7", 2093.004, 96), // 96
    ("C#7/Db7", 2217.461, 97),
    ("D7", 2349.318, 98),
    ("D#7/Eb7", 2489.015, 99),
    ("E7", 2_637.02, 100),
    ("F7", 2793.825, 101),
    ("F#7/Gb7", 2959.955, 102),
    ("G7", 3135.963, 103),
    ("G#7/Ab7", 3322.437, 104),
    ("A7", 3520.000, 105),
    ("A#7/Bb7", 3_729.31, 106),
    ("B7", 3951.066, 107),
    ("C8", 4186.009, 108),
    ("C#8/Db8", 4434.922, 109),
    ("D8", 4698.636, 110),
    ("D#8/Eb8", 4978.031, 111),
    ("E8", 5_274.04, 112),
    ("F8", 5587.651, 113),
    ("F#8/Gb8", 5_919.91, 114),
    ("G8", 6271.927, 115),
    ("G#8/Ab8", 6644.875, 116),
    ("A8", 7040.000, 117),
    ("A#8/Bb8", 7_458.62, 118),
    ("B8", 7902.132, 119),
    ("C9", 8372.018, 120),
    ("C#9/Db9", 8869.844, 121),
    ("D9", 9397.272, 122),
    ("D#9/Eb9", 9956.063, 123),
    ("E9", 10548.081, 124),
    ("F9", 11175.303, 125),
    ("F#9/Gb9", 11839.821, 126),
    ("G9", 12543.854, 127),
];

#[derive(Default, Clone, Debug, PartialEq)]
pub enum ArpeggiatorType {
    #[default]
    NoteOrder,
    Randomize,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Arpeggiator {
    sequence: Vec<u16>,
    sequence_index: usize,
}

impl Arpeggiator {
    pub fn new(sequence: Vec<u16>) -> Arpeggiator {
        Arpeggiator {
            sequence,
            sequence_index: 0,
        }
    }

    pub fn add_note(&mut self, note_number: u16) {
        if self.sequence.len() == 1 && self.sequence[0] == FIRST_REST_NOTE {
            self.sequence.push(note_number);
            self.sequence.remove(0);
        }

        if !self.sequence.contains(&note_number) {
            self.sequence.push(note_number);
        }
    }

    pub fn remove_note(&mut self, note_number: u16) {
        if self.sequence.len() == 1 {
            self.sequence.push(FIRST_REST_NOTE);
            self.sequence.remove(0);
        }

        if let Some(note_index) = self.sequence.iter().position(|&note| note == note_number) {
            self.sequence.remove(note_index);
        }
    }

    pub fn next_midi_note(&mut self, state: ArpeggiatorType) -> u16 {
        if self.sequence_index < self.sequence.len() - 1 {
            self.sequence_index += 1;
        } else {
            self.sequence_index = 0;
        }

        match state {
            ArpeggiatorType::NoteOrder => self.sequence[self.sequence_index],
            ArpeggiatorType::Randomize => {
                let index = rand::rng().random_range(0..self.sequence.len());
                self.sequence[index]
            }
        }
    }

    pub fn get_frequency_from_midi_note(&self, midi_note: u16) -> f32 {
        if midi_note >= NUMBER_OF_MIDI_NOTES as u16 {
            return REST_FREQUENCY;
        }

        MIDI_NOTE_FREQUENCIES[midi_note as usize].1
    }
}
