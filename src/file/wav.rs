use crate::{
    Balance, Error, LoopRange,
    generator::{OneShotNode, SampleLoopNode},
};
use hound::WavReader;
use soundfont::data::SampleHeader;

use std::io::Cursor;

/// Make a WavSource. The source note is a MIDI notes, where 69 is A440.
pub fn wav_from_file(
    file_name: &str,
    source_note: u8,
    loop_range: Option<LoopRange>,
    balance: Balance,
    node_id: Option<u64>,
) -> Result<SampleLoopNode, Error> {
    let wav = WavReader::open(file_name)?;
    let spec = wav.spec();
    let data: Vec<f32> = wav.into_samples().map(|s| s.unwrap()).collect();
    SampleLoopNode::new_from_data(spec, source_note, balance, data, loop_range, node_id)
}

/// Make a WavSource. The source note is a MIDI note, where 69 is A440.
pub fn wav_from_bytes(
    bytes: &[u8],
    source_note: u8,
    loop_range: Option<LoopRange>,
    balance: Balance,
    node_id: Option<u64>,
) -> Result<SampleLoopNode, Error> {
    let cursor = Cursor::new(bytes);
    let wav = WavReader::new(cursor)?;
    let spec = wav.spec();
    let data: Vec<f32> = wav.into_samples().map(|s| s.unwrap()).collect();
    SampleLoopNode::new_from_data(spec, source_note, balance, data, loop_range, node_id)
}

pub fn wav_from_i16_samples(
    header: &SampleHeader,
    balance: Balance,
    source_data: &[i16],
) -> Result<SampleLoopNode, Error> {
    let mut data: Vec<f32> = vec![0.0; source_data.len()];
    for (i, sample) in source_data.iter().enumerate() {
        data[i] = *sample as f32 / 32768.0;
    }
    SampleLoopNode::new_from_raw_sf2_data(header, balance, data)
}

pub fn one_shot_from_file(
    file_name: &str,
    balance: Balance,
    node_id: Option<u64>,
) -> Result<OneShotNode, Error> {
    let wav = WavReader::open(file_name)?;
    let spec = wav.spec();
    let data: Vec<f32> = wav.into_samples().map(|s| s.unwrap()).collect();
    OneShotNode::new_from_data(spec, balance, data, node_id)
}

pub fn one_shot_from_bytes(
    bytes: &[u8],
    balance: Balance,
    node_id: Option<u64>,
) -> Result<OneShotNode, Error> {
    let cursor = Cursor::new(bytes);
    let wav = WavReader::new(cursor)?;
    let spec = wav.spec();
    let data: Vec<f32> = wav.into_samples().map(|s| s.unwrap()).collect();
    OneShotNode::new_from_data(spec, balance, data, node_id)
}

pub fn one_shot_from_i16_samples(
    header: &SampleHeader,
    balance: Balance,
    source_data: &[i16],
) -> Result<OneShotNode, Error> {
    let mut data: Vec<f32> = vec![0.0; source_data.len()];
    for (i, sample) in source_data.iter().enumerate() {
        data[i] = *sample as f32 / 32768.0;
    }
    OneShotNode::new_from_raw_sf2_data(header, balance, data)
}
