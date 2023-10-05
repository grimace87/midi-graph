
use crate::{MidiProcessor, SquareAudio};
use std::time::Duration;
use cpal::traits::StreamTrait;
use wasm_bindgen::prelude::*;

const MIDI_FILE: &'static [u8] = include_bytes!("../resources/MIDI_sample.mid");

#[wasm_bindgen]
pub fn play_stream() {
    let smf = MidiProcessor::from_bytes(MIDI_FILE).unwrap();
    let streamer = SquareAudio::default();
    let stream = smf.open_stream(streamer).expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::sleep(Duration::from_secs(5));
    stream.pause().expect("Could not pause the stream");
}