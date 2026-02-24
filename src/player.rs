use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

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
