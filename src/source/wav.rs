use crate::{
    consts, util, BufferConsumer, BufferConsumerNode, Error, LoopRange, Node, NodeEvent, NoteEvent,
    Status,
};
use hound::{SampleFormat, WavSpec};
use soundfont::data::{sample::SampleLink, SampleHeader};

pub struct WavSource {
    is_on: bool,
    source_note: u8,
    source_channel_count: usize,
    loop_start_data_position: usize,
    loop_end_data_position: usize,
    data_position: usize,
    current_note: u8,
    source_data: Vec<f32>,
    playback_scale: f64,
}

impl WavSource {
    pub fn new_from_raw_sf2_data(header: &SampleHeader, data: Vec<f32>) -> Result<Self, Error> {
        Self::validate_header(header)?;
        let source_channel_count = match header.sample_type {
            SampleLink::MonoSample => 1,
            _ => {
                return Err(Error::User(format!(
                    "SF2: Unsupported sample type: {:?}",
                    header.sample_type
                )));
            }
        };
        let sample_offset = header.start as usize;
        let loop_range = Some(LoopRange::new_frame_range(
            (header.loop_start as usize - sample_offset) / source_channel_count,
            (header.loop_end as usize - sample_offset) / source_channel_count,
        ));
        Self::validate_loop_range(&data, source_channel_count, &loop_range)?;
        let loop_range = loop_range.unwrap();
        Ok(Self::new(
            header.sample_rate,
            source_channel_count,
            header.origpitch,
            loop_range,
            data,
        ))
    }

    /// Make a new WavSource holding the given sample data.
    /// Data in the spec will be checked for compatibility.
    /// The note is a MIDI key, where A440 is 69.
    pub fn new_from_data(
        spec: WavSpec,
        source_note: u8,
        data: Vec<f32>,
        loop_range: Option<LoopRange>,
    ) -> Result<Self, Error> {
        Self::validate_spec(&spec)?;
        Self::validate_loop_range(&data, spec.channels as usize, &loop_range)?;
        let loop_range = match loop_range {
            Some(range) => range,
            None => LoopRange::new_frame_range(0, usize::MAX / spec.channels as usize),
        };
        Ok(Self::new(
            spec.sample_rate,
            spec.channels as usize,
            source_note,
            loop_range,
            data,
        ))
    }

    fn new(
        sample_rate: u32,
        channels: usize,
        source_note: u8,
        loop_range: LoopRange,
        data: Vec<f32>,
    ) -> Self {
        let playback_scale = consts::PLAYBACK_SAMPLE_RATE as f64 / sample_rate as f64;
        Self {
            is_on: false,
            source_note,
            source_channel_count: channels,
            loop_start_data_position: loop_range.start_frame * channels,
            loop_end_data_position: loop_range.end_frame * channels,
            data_position: data.len(),
            current_note: 0,
            source_data: data,
            playback_scale,
        }
    }

    fn validate_header(header: &SampleHeader) -> Result<(), Error> {
        match header.sample_type {
            SampleLink::MonoSample => Ok(()),
            _ => Err(Error::User(format!(
                "SF2: Unsupported sample type: {:?}",
                header.sample_type
            ))),
        }
    }

    fn validate_loop_range(
        data: &Vec<f32>,
        channel_count: usize,
        loop_range: &Option<LoopRange>,
    ) -> Result<(), Error> {
        let Some(range) = loop_range else {
            return Ok(());
        };
        let frames_in_data = data.len() / channel_count;
        let range_makes_sense =
            range.start_frame <= frames_in_data || range.end_frame > frames_in_data;
        if !range_makes_sense {
            return Err(Error::User(format!(
                "Invalid sample loop range: {} to {}",
                range.start_frame, range.end_frame
            )));
        }
        Ok(())
    }

    fn validate_spec(spec: &WavSpec) -> Result<(), Error> {
        if spec.channels == 0 || spec.channels > 2 {
            return Err(Error::User(format!(
                "{} channels is not supported",
                spec.channels
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

    fn stretch_buffer(
        src: &[f32],
        src_channels: usize,
        dst: &mut [f32],
        source_frames_per_output_frame: f64,
    ) -> (usize, usize) {
        let mut src_index = 0;
        let mut dst_index = 0;
        while src_index < src.len() && dst_index < dst.len() {
            match src_channels {
                1 => {
                    let sample = src[src_index];
                    dst[dst_index] += sample;
                    dst[dst_index + 1] += sample;
                }
                2 => {
                    dst[dst_index] += src[src_index];
                    dst[dst_index + 1] += src[src_index + 1];
                }
                _ => {}
            }
            dst_index += 2;
            src_index =
                ((dst_index / 2) as f64 * source_frames_per_output_frame) as usize * src_channels;
        }
        let src_data_points_advanced = src_index;
        let dst_data_points_advanced = dst_index;
        (src_data_points_advanced, dst_data_points_advanced)
    }
}

impl BufferConsumerNode for WavSource {}

impl Node for WavSource {
    fn on_event(&mut self, event: &NodeEvent) {
        match event {
            NodeEvent::Note { note, event } => match event {
                NoteEvent::NoteOn { vel: _ } => {
                    self.is_on = true;
                    self.data_position = 0;
                    self.current_note = *note;
                }
                NoteEvent::NoteOff { vel: _ } => {
                    if self.current_note != *note || !self.is_on {
                        return;
                    }
                    self.is_on = false;
                }
            },
            NodeEvent::Control {
                node_id: _,
                event: _,
            } => {}
        }
    }
}

impl BufferConsumer for WavSource {
    fn duplicate(&self) -> Result<Box<dyn BufferConsumerNode + Send + 'static>, Error> {
        let sample_rate = (consts::PLAYBACK_SAMPLE_RATE as f64 / self.playback_scale) as u32;
        let loop_range = LoopRange::new_frame_range(
            self.loop_start_data_position / self.source_channel_count,
            self.loop_end_data_position / self.source_channel_count,
        );
        let source = Self::new(
            sample_rate,
            self.source_channel_count,
            self.source_note,
            loop_range,
            self.source_data.clone(),
        );
        Ok(Box::new(source))
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) -> Status {
        if buffer.is_empty() {
            return Status::Ok;
        }

        if self.is_on && self.data_position >= self.loop_end_data_position {
            self.data_position -= self.loop_end_data_position - self.loop_start_data_position;
        }

        // Scaling
        let relative_pitch =
            util::relative_pitch_ratio_of(self.current_note, self.source_note) as f64;
        let source_frames_per_output_frame = relative_pitch * self.playback_scale;

        #[cfg(debug_assertions)]
        assert_eq!(buffer.len() % consts::CHANNEL_COUNT, 0);

        let mut remaining_buffer = &mut buffer[0..];
        while remaining_buffer.len() > 0 {
            if self.data_position >= self.source_data.len() {
                self.is_on = false;
                return Status::Ended;
            }

            let source_end_point = match self.is_on {
                true => self.source_data.len().min(self.loop_end_data_position),
                false => self.source_data.len(),
            };

            let (src_data_points_advanced, dst_data_points_advanced) = Self::stretch_buffer(
                &self.source_data[self.data_position..source_end_point],
                self.source_channel_count,
                remaining_buffer,
                source_frames_per_output_frame,
            );

            self.data_position += src_data_points_advanced;

            if self.data_position != source_end_point {
                break;
            }
            if self.is_on && source_end_point == self.loop_end_data_position {
                self.data_position = self.loop_start_data_position;
                let remaining_dst_data_points = remaining_buffer.len() - dst_data_points_advanced;
                let dst_buffer_index = buffer.len() - remaining_dst_data_points;
                remaining_buffer = &mut buffer[dst_buffer_index..];
            } else {
                self.is_on = false;
                return Status::Ended;
            }
        }

        Status::Ok
    }
}
