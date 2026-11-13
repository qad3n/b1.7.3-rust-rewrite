use std::fs::File;
use std::io::{BufWriter, ErrorKind, Write};
use std::num::{NonZeroU8, NonZeroU32};
use std::path::Path;
use symphonia::core::audio::{AudioBufferRef, SampleBuffer};
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::errors::Error as AudioError;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use vorbis_rs::{VorbisBitrateManagementStrategy, VorbisEncoder, VorbisEncoderBuilder};

// vorbis quality 0.4 is about 128 kbps for stereo music and sounds like the source
const QUALITY: f32 = 0.4;
// one stream per file so any fixed serial is valid and keeps the output the same for the same input
const SERIAL: i32 = 1;
const MAX_CHANNELS: usize = 2;

type Encoder = VorbisEncoder<BufWriter<File>>;

// decodes whatever the site served and writes it as ogg vorbis, the one format the sound engine plays
pub fn compress(source: &Path, song: &Path) -> Result<(), String>
{
    let file: File = File::open(source).map_err(|error| format!("could not open the download: {error}"))?;
    let stream: MediaSourceStream = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint: Hint = Hint::new();
    if let Some(extension) = source.extension().and_then(|extension| extension.to_str())
    {
        hint.with_extension(extension);
    }

    let mut format: Box<dyn FormatReader> = symphonia::default::get_probe()
        .format(&hint, stream, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|error| format!("the download is not audio this game can read: {error}"))?
        .format;
    let track: &symphonia::core::formats::Track = format.default_track().ok_or("the download has no audio")?;
    let trackId: u32 = track.id;
    let mut decoder: Box<dyn Decoder> = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|error| format!("the site served audio this game cannot decode: {error}"))?;

    let mut encoder: Option<Encoder> = None;
    loop
    {
        let packet: symphonia::core::formats::Packet = match format.next_packet()
        {
            Ok(packet) => packet,
            Err(AudioError::IoError(error)) if error.kind() == ErrorKind::UnexpectedEof => break,
            Err(error) => return Err(format!("could not read the download: {error}")),
        };

        if packet.track_id() != trackId
        {
            continue;
        }

        match decoder.decode(&packet)
        {
            Ok(decoded) => append(&mut encoder, decoded, song)?,
            Err(AudioError::DecodeError(reason)) =>
            {
                crate::log::debug(crate::log::LOADER, &format!("skipped a damaged audio packet: {reason}"));
            }
            Err(error) => return Err(format!("could not decode the download: {error}")),
        }
    }

    let encoder: Encoder = encoder.ok_or("the download held no sound")?;
    let mut sink: BufWriter<File> = encoder.finish().map_err(|error| format!("could not finish the song: {error}"))?;
    sink.flush().map_err(|error| format!("could not write the song: {error}"))
}

// the encoder starts with the first decoded block since some formats only state their channels there
fn append(encoder: &mut Option<Encoder>, decoded: AudioBufferRef, song: &Path) -> Result<(), String>
{
    let frames: usize = decoded.frames();
    if frames == 0
    {
        return Ok(());
    }

    let rate: u32 = decoded.spec().rate;
    let channels: usize = decoded.spec().channels.count().min(MAX_CHANNELS);
    let mut samples: SampleBuffer<f32> = SampleBuffer::new(decoded.capacity() as u64, *decoded.spec());
    samples.copy_planar_ref(decoded);

    let planes: Vec<&[f32]> = samples.samples().chunks(frames).take(channels).collect();
    let encoder: &mut Encoder = match encoder
    {
        Some(encoder) => encoder,
        None => encoder.insert(start(song, rate, channels)?),
    };

    encoder.encode_audio_block(&planes).map_err(|error| format!("could not encode the song: {error}"))
}

fn start(song: &Path, rate: u32, channels: usize) -> Result<Encoder, String>
{
    let sink: BufWriter<File> =
        BufWriter::new(File::create(song).map_err(|error| format!("could not make the song file: {error}"))?);
    let rate: NonZeroU32 = NonZeroU32::new(rate).ok_or("the audio has no sample rate")?;
    let channels: NonZeroU8 =
        u8::try_from(channels).ok().and_then(NonZeroU8::new).ok_or("the audio has no channels")?;

    VorbisEncoderBuilder::new_with_serial(rate, channels, sink, SERIAL)
        .bitrate_management_strategy(VorbisBitrateManagementStrategy::QualityVbr { target_quality: QUALITY })
        .build()
        .map_err(|error| format!("could not start the encoder: {error}"))
}
