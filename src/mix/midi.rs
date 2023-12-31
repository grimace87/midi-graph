
use crate::{AudioStreamer, Error};
use cpal::{Stream, StreamConfig};
use cpal::traits::{DeviceTrait, HostTrait};
use midly::Smf;

pub struct MidiProcessor {
    smf: Smf<'static>
}

impl MidiProcessor {

    pub fn from_file(smf: Smf<'static>) -> MidiProcessor {
        Self { smf }
    }

    pub fn open_stream<S>(self, mut streamer: S) -> Result<Stream, Error> where S: AudioStreamer + Send + 'static {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or(Error::NoDevice)?;
        let required_config = StreamConfig {
            buffer_size: cpal::BufferSize::Fixed(2048),
            channels: 2,
            sample_rate: cpal::SampleRate(48000)
        };
        let stream = device.build_output_stream(
            &required_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                streamer.fill_buffer(data);
            },
            move |err| {
                println!("Stream error: {:?}", err);
            },
            None
        )?;
        Ok(stream)
    }
}
