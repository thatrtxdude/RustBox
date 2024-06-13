// player.rs
use lofty::{Accessor, AudioFile, Probe, TaggedFileExt};
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::discord::update_discord_activity;
use crate::ui::run_ui;
use std::borrow::Cow;

pub fn play_music(file_path: &str, repeat: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Probe file, check if path is valid or file corrupted
    let tagged_file = Probe::open(file_path)
        .expect("ERROR: No path")
        .read()
        .expect("ERROR: Failed to read file");

    // Check for tags
    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag())
        .ok_or("No tags found")?;

    // Metadata
    let title = tag.title().unwrap_or(Cow::Borrowed("None"));
    let artist = tag.artist().unwrap_or(Cow::Borrowed("None"));
    let bitrate = tagged_file.properties().audio_bitrate();
    let overall_bitrate = tagged_file.properties().overall_bitrate();
    let duration = tagged_file.properties().duration();
    let display_duration = format!("{:02}:{:02}", duration.as_secs() / 60, duration.as_secs() % 60);

    println!("Playing {} by {}", title, artist);

    // Create an output stream and a sink for audio playback
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle).unwrap()));

    // Load and play the file
    if let Ok(file) = File::open(file_path) {
        let source = Decoder::new(BufReader::new(file)).unwrap();
        sink.lock().unwrap().append(source);
    } else {
        eprintln!("Error: Failed to open file {}", file_path);
        return Ok(());
    }

    // Update Discord activity
    let format_title = format!("Playing {} by {}", title, artist);
    update_discord_activity(&format_title)?;

    // Handle user inputs
    let sink_clone = Arc::clone(&sink);
    let bitrate_clone = bitrate.clone();
    let overall_bitrate_clone = overall_bitrate.clone();
    let display_duration_clone = display_duration.clone();

    thread::spawn(move || {
        run_ui(sink_clone, bitrate_clone, overall_bitrate_clone, display_duration_clone);
    });

    // Loop if repeat is present
    loop {
        if sink.lock().unwrap().empty() {
            if repeat {
                if let Ok(file) = File::open(file_path) {
                    let source = Decoder::new(BufReader::new(file)).unwrap();
                    sink.lock().unwrap().append(source);
                }
            } else {
                return Ok(());
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}
