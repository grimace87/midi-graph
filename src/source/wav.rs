use crate::{AudioSource, Error};
use hound::{SampleFormat, WavSpec};

pub struct WavSource {
    position: usize,
    data: Vec<f32>,
}

impl WavSource {
    pub fn new_from_data(spec: WavSpec, data: Vec<f32>) -> Result<Self, Error> {
        Self::validate_spec(&spec)?;
        Ok(Self { position: 0, data })
    }

    fn validate_spec(spec: &WavSpec) -> Result<(), Error> {
        if spec.channels != 1 {
            return Err(Error::User(format!(
                "{} channels is not supported",
                spec.channels
            )));
        }
        if spec.sample_rate != 48000 {
            return Err(Error::User(format!(
                "{} samples per second is not supported",
                spec.sample_rate
            )));
        }
        if spec.sample_format != SampleFormat::Float {
            return Err(Error::User(format!(
                "Sample format {:?} is not supported",
                spec.sample_format
            )));
        }
        if spec.bits_per_sample != 32 {
            return Err(Error::User(format!(
                "{} bits per sample is not supported",
                spec.bits_per_sample
            )));
        }
        Ok(())
    }
}

impl AudioSource for WavSource {
    fn on_note_on(&mut self, key: u8) {
        self.position = 0;
    }

    fn on_note_off(&mut self, key: u8) {
        self.position = self.data.len();
    }

    fn fill_buffer(&mut self, key: u8, buffer: &mut [f32]) {
        let relative_pitch = crate::util::relative_pitch_of(key);
        let size = buffer.len();
        let samples_remaining = self.data.len() - self.position;
        if samples_remaining == 0 {
        } else if samples_remaining < size {
            let source = &self.data[self.position..(self.position + samples_remaining)];
            for i in 0..samples_remaining {
                buffer[i] += source[i];
            }
        } else {
            let source = &self.data[self.position..(self.position + size)];
            for i in 0..size {
                buffer[i] += source[i];
            }
        }
        self.position = (self.position + size).min(self.data.len());
    }
}
