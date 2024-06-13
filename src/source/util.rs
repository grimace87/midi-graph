// Get pitch of a MIDI note in terms of semitones relative to A440
#[inline]
pub fn relative_pitch_of(key: u8) -> f32 {
    key as f32 - 69.0
}

// Get frequency of a MIDI note
#[inline]
pub fn frequency_of(key: u8) -> f32 {
    let relative_pitch = relative_pitch_of(key);
    440.0 * 2.0f32.powf(relative_pitch / 12.0)
}
