use serde_derive::Deserialize;
use std::collections::HashMap;

const fn default_amplitude() -> f32 {
    0.5
}

const fn default_duty_cycle() -> f32 {
    0.5
}

const fn default_note_for_16_shifts() -> u8 {
    64
}

const fn default_attack() -> f32 {
    0.125
}

const fn default_decay() -> f32 {
    0.25
}

const fn default_sustain() -> f32 {
    0.5
}

const fn default_release() -> f32 {
    0.125
}

#[derive(Deserialize)]
pub struct Config {
    pub midi: MidiDataSource,
    pub channels: HashMap<usize, FontSource>,
}

#[derive(Deserialize)]
pub enum MidiDataSource {
    FilePath(String),
}

#[derive(Deserialize)]
pub enum FontSource {
    Ranges(Vec<RangeSource>),
    Sf2FilePath {
        path: String,
        instrument_index: usize,
    },
}

#[derive(Deserialize)]
pub struct RangeSource {
    pub source: SoundSource,
    pub lower: u8,
    pub upper: u8,
}

/// Loop range, defined as the inclusive start and exclusive end.
/// These points are specified in frames, not data points.
#[derive(Deserialize)]
pub struct Loop {
    pub start: usize,
    pub end: usize,
}

#[derive(Deserialize)]
pub enum SoundSource {
    SquareWave {
        #[serde(default = "default_amplitude")]
        amplitude: f32,
        #[serde(default = "default_duty_cycle")]
        duty_cycle: f32,
    },
    TriangleWave {
        #[serde(default = "default_amplitude")]
        amplitude: f32,
    },
    SawtoothWave {
        #[serde(default = "default_amplitude")]
        amplitude: f32,
    },
    LfsrNoise {
        #[serde(default = "default_amplitude")]
        amplitude: f32,
        inside_feedback: bool,
        #[serde(default = "default_note_for_16_shifts")]
        note_for_16_shifts: u8,
    },
    SampleFilePath {
        path: String,
        base_note: u8,
        looping: Option<Loop>,
    },
    Envelope {
        #[serde(default = "default_attack")]
        attack_time: f32,
        #[serde(default = "default_decay")]
        decay_time: f32,
        #[serde(default = "default_sustain")]
        sustain_multiplier: f32,
        #[serde(default = "default_release")]
        release_time: f32,
        source: Box<SoundSource>,
    },
}
