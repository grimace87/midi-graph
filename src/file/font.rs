use crate::{
    Balance, DebugLogging, Error, GraphNode, NoteRange,
    file::wav::wav_from_i16_samples,
    group::{FontNode, FontNodeBuilder, PolyphonyNode},
};
use byteorder::{LittleEndian, ReadBytesExt};
use soundfont::{
    SfEnum, SoundFont2, Zone,
    data::{GeneratorAmount, GeneratorType},
};
use std::{
    fs::File,
    io::{BufReader, Cursor, Read, Seek, SeekFrom},
};

pub fn soundfont_from_file(
    node_id: Option<u64>,
    file_name: &str,
    instrument_index: usize,
    polyphony_voices: usize,
) -> Result<FontNode, Error> {
    let file = File::open(file_name)?;
    let reader = BufReader::new(file);
    soundfont_from_reader(reader, node_id, instrument_index, polyphony_voices)
}

pub fn soundfont_from_bytes(
    node_id: Option<u64>,
    bytes: &[u8],
    instrument_index: usize,
    polyphony_voices: usize,
) -> Result<FontNode, Error> {
    let cursor = Cursor::new(bytes);
    soundfont_from_reader(cursor, node_id, instrument_index, polyphony_voices)
}

fn soundfont_from_reader<R>(
    mut reader: R,
    node_id: Option<u64>,
    instrument_index: usize,
    polyphony_voices: usize,
) -> Result<FontNode, Error>
where
    R: Read + Seek,
{
    let sf2 = SoundFont2::load(&mut reader)?;
    validate_sf2_file(&sf2)?;

    if DebugLogging::get_log_on_init() {
        log_opened_sf2(&sf2);
    }

    let sample_chunk_metadata = &sf2
        .sample_data
        .smpl
        .ok_or_else(|| Error::User("There was no sample header in the SF2 file".to_owned()))?;
    let Some(instrument) = sf2.instruments.get(instrument_index) else {
        return Err(Error::User(format!(
            "Index {} is out of bounds ({} instruments in the SF2 file)",
            instrument_index,
            sf2.instruments.len()
        )));
    };

    if DebugLogging::get_log_on_init() {
        println!("SF2: Using instrument from file: {:?}", &instrument.header);
    }

    let mut soundfont_builder = FontNodeBuilder::new(node_id);
    for zone in instrument.zones.iter() {
        let Some(sample_index) = zone.sample() else {
            println!("WARNING: SF2: Sample index not found for instrument zone");
            continue;
        };
        let Some(sample_header) = sf2.sample_headers.get(*sample_index as usize) else {
            println!(
                "WARNING: SF2: Sample index {} not found matching instrument zone",
                sample_index
            );
            continue;
        };

        let sample_file_offset = sample_chunk_metadata.offset + sample_header.start as u64;
        let sample_length = sample_header.end as u64 - sample_file_offset;
        let sample_data = load_sample(&mut reader, sample_file_offset, sample_length)?;
        let note_range = note_range_for_zone(zone)?;
        let source = wav_from_i16_samples(sample_header, Balance::Both, &sample_data)?;

        let polyphony: GraphNode = match polyphony_voices {
            0 | 1 => {
                let polyphony = PolyphonyNode::new(None, polyphony_voices, Box::new(source))?;
                Box::new(polyphony)
            }
            _ => Box::new(source),
        };

        soundfont_builder = soundfont_builder.add_range(note_range, polyphony)?;
    }
    Ok(soundfont_builder.build())
}

fn validate_sf2_file(sf2: &SoundFont2) -> Result<(), Error> {
    if sf2.info.version.major != 2 {
        return Err(Error::User(format!(
            "Unsupported SF2 file version {}; only version 2 is supported",
            sf2.info.version.major
        )));
    }

    if !sf2.presets.is_empty() {
        println!("WARNING: SF2: File has presets; these will be ignored");
    }
    if sf2.instruments.is_empty() {
        return Err(Error::User("The SF2 file has no instruments".to_owned()));
    }
    Ok(())
}

fn load_sample<R>(
    reader: &mut R,
    sample_position: u64,
    sample_length: u64,
) -> Result<Vec<i16>, Error>
where
    R: Read + Seek,
{
    let byte_size = std::mem::size_of::<i16>();
    reader.seek(SeekFrom::Start(sample_position * byte_size as u64))?;
    let mut sample_data = vec![0i16; sample_length as usize];
    reader.read_i16_into::<LittleEndian>(&mut sample_data)?;
    Ok(sample_data)
}

fn note_range_for_zone(zone: &Zone) -> Result<NoteRange, Error> {
    for generator in zone.gen_list.iter() {
        if let SfEnum::Value(GeneratorType::KeyRange) = generator.ty {
            if let GeneratorAmount::Range(range) = generator.amount {
                return Ok(NoteRange::new_inclusive_range(range.low, range.high));
            }
        }
    }
    Err(Error::User(
        "No key range found in an instrument zone in the SF2 file".to_owned(),
    ))
}

fn log_opened_sf2(sf2: &SoundFont2) {
    println!(
        "SF2: Contains {} presets, {} instruments and {} samples",
        sf2.presets.len(),
        sf2.instruments.len(),
        sf2.sample_headers.len()
    );
}
