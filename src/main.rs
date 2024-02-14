extern crate clap;
extern crate lofty;
extern crate rodio;
extern crate discord_rich_presence;

use clap::{App, Arg};
use lofty::{Accessor, Probe, AudioFile, TaggedFileExt};
use rodio::Sink;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

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
                .help("Input file path to file that the player should play"),
        )
        .arg(Arg::with_name("repeat").long("repeat").help("Repeats song"))
        .get_matches();

    // file path
    let file_path = matches.value_of("file").unwrap();

    // check if repeat argument is present
    let repeat = matches.is_present("repeat");

    // probe file, check if path is valid or file corrupted
    let tagged_file = Probe::open(file_path)
        .expect("ERROR: No path")
        .read()
        .expect("ERROR: Failed to read file");

    // check for tags
    let tag = match tagged_file.primary_tag() {
        Some(primary_tag) => primary_tag,
        None => match tagged_file.first_tag(){
            Some(first_tag) => first_tag,
            None => {
                println!("No tags found");
                return Ok(())
            }
        },
    };

    // test for new git key, will be removed later 

    // bunch of metadata related stuff
    let binding_title = tag.title(); // bindings so value doesn't get dropped while let title is borrowing
    let binding_artist = tag.artist(); // same thing here but for artist

    let title = binding_title.as_deref().unwrap_or("None"); // set title
    let artist = binding_artist.as_deref().unwrap_or("None"); // set artist

    let bitrate = tagged_file.properties().audio_bitrate(); // audio bitrate
    let overall_bitrate = tagged_file.properties().overall_bitrate(); //  overall file bitrate

    let duration = tagged_file.properties().duration(); // song duration
    let seconds = duration.as_secs() % 60; // convert to seconds

    let displayduration = format!("{:02}:{:02}", (duration.as_secs() - seconds) / 60, seconds); // format duration so its readable

    println!("Playing {} by {}", title, artist);
    
    // Create an output stream
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    // Create a sink for audio playback
    let sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle).unwrap()));

    // Load and play the FLAC file
     if let Ok(file) = std::fs::File::open(file_path) {
        let source = rodio::Decoder::new(std::io::BufReader::new(file)).unwrap();
        sink.lock().unwrap().append(source);
    } else {
        eprintln!("Error: Failed to open file {}", file_path);
        return Ok(());
    };
    
    // format title for discord rpc
    let format_title = format!("Playing {} by {}", title, artist);

    let mut client = DiscordIpcClient::new("1206034100977406012")?;

    client.connect()?;

    let payload = activity::Activity::new().state("Listening to music").details(&format_title);

    client.set_activity(payload)?;
    
    // clone for handle
    let sink_clone = Arc::clone(&sink);
    let bitrate_clone = bitrate.clone();
    let ovbitrate_clone = overall_bitrate.clone();
    let displayduration_clone = displayduration.clone();
    
    // handle user inputs    
     thread::spawn(move || loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let sink = sink_clone.lock().unwrap();
        match input.trim() {
            "play" => sink.play(),
            "pause" => sink.pause(),
            "bitrate" => println!("Audio Bitrate: {}, Overall Bitrate: {}", bitrate_clone.unwrap_or(0), ovbitrate_clone.unwrap_or(0)),
            "duration" => println!("{}", displayduration_clone),
            "help" => println!("Available inputs: play, pause, bitrate, duration | Available arguments: --repeat"),
            _ => (),
        }
    });

    // loop if repeat is present
    loop {
        if sink.lock().unwrap().empty() {
            if repeat {
                if let Ok(file) = std::fs::File::open(file_path) {
                    let source = rodio::Decoder::new(std::io::BufReader::new(file)).unwrap();
                    sink.lock().unwrap().append(source);
                }
            } else {
                client.close(); // kill discord client
                std::process::exit(0) // wow
            }

         }
    }
}
