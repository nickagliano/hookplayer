use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

// PORT: PLAYER
// Swap this function to change the audio backend. Only the signature is contractual.
// Default: rodio (cross-platform, no system deps). Alternatives: afplay via Command (macOS only),
// symphonia for broader format support, or any crate that can play a file at a given volume.
pub fn play(path: &Path, volume: f32) -> Result<(), Box<dyn std::error::Error>> {
    let file = BufReader::new(File::open(path)?);
    let source = Decoder::new(file)?;

    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    sink.set_volume(volume);
    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}
