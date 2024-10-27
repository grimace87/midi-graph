// Test suite for the Web and headless browsers.
#[cfg(target_arch = "wasm32")]
#[cfg(test)]
mod wasm_tests;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests;

#[cfg(target_arch = "wasm32")]
mod wasm_demo;

mod config;
mod error;
mod file;
mod mix;
mod source;

pub use config::{Config, FontSource, Loop, MidiDataSource, RangeSource, SoundSource};
pub use error::Error;
pub use mix::base::BaseMixer;
pub use source::{
    async_receiver::AsyncEventReceiver,
    envelope::Envelope,
    fader::Fader,
    font::{SoundFont, SoundFontBuilder},
    midi::{track::MidiTrackSource, MidiSource, MidiSourceBuilder},
    mixer::MixerSource,
    noise::LfsrNoiseSource,
    one_shot::OneShotSource,
    sawtooth::SawtoothWaveSource,
    square::SquareWaveSource,
    triangle::TriangleWaveSource,
    wav::WavSource,
    BufferConsumer, BufferConsumerNode, ControlEvent, LoopRange, Node, NodeEvent, NoteEvent,
    NoteRange,
};

pub mod util {
    pub use crate::file::config::*;
    pub use crate::file::font::*;
    pub use crate::file::midi::*;
    pub use crate::file::wav::*;
    pub use crate::source::midi::util::*;
    pub use crate::source::util::*;
}

pub mod consts {
    pub const PLAYBACK_SAMPLE_RATE: usize = 48000;
    pub const CHANNEL_COUNT: usize = 2;
    pub const BUFFER_SIZE: usize = 2048;
}
