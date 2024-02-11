extern crate clap;
extern crate lofty;
extern crate rodio;

use clap::{App, Arg};
use lofty::{Accessor, Probe, TaggedFileExt};
use rodio::Sink;
use std::fs::File;

use std::sync::{Arc, Mutex};
use std::thread;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let matches = App::new("RustCLIMusic")
        .arg(
            Arg::with_name("file")
                .required(true)
                .takes_value(true)
                .index(1)
                .help("Input FLAC file to play"),
        )
        .arg(Arg::with_name("repeat").long("repeat").help("Repeats song"))
        .get_matches();

    let file_path = matches.value_of("file").unwrap();
    let repeat = matches.is_present("repeat");

    let tagged_file = Probe::open(file_path)
        .expect("ERROR: No path")
        .read()
        .expect("ERROR: Failed to read file");

    let tag = match tagged_file.primary_tag() {
        Some(primary_tag) => primary_tag,
        None => tagged_file.first_tag().expect("ERROR: No tags found"),
    };

    let binding_title = tag.title();
    let binding_artist = tag.artist();

    let title = binding_title.as_deref().unwrap_or("None");
    let artist = binding_artist.as_deref().unwrap_or("None");

    // Create an output stream
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    // Create a sink for audio playback
    let sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle).unwrap()));

    // Load and play the FLAC file
    let _source = if let Ok(file) = std::fs::File::open(file_path) {
        let source = rodio::Decoder::new(std::io::BufReader::new(file)).unwrap();
        sink.lock().unwrap().append(source);
        println!("Playing {} by {}", title, artist);
    } else {
        eprintln!("Error: Failed to open file {}", file_path);
        return Ok(());
    };

    let format_title = format!("Playing {} by {}", title, artist);

    let sink_clone = Arc::clone(&sink);
    let handle = thread::spawn(move || loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let sink = sink_clone.lock().unwrap();
        match input.trim() {
            "play" => sink.play(),
            "pause" => sink.pause(),
            _ => (),
        }
    });

    loop {
        if sink.lock().unwrap().empty() {
            if repeat {
                if let Ok(file) = std::fs::File::open(file_path) {
                    let source = rodio::Decoder::new(std::io::BufReader::new(file)).unwrap();
                    sink.lock().unwrap().append(source);
                }
            } else {
                break;
            }
        }
    }
    handle.join().unwrap();

    Ok(())
}
