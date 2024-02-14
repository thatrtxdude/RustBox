extern crate clap;
extern crate lofty;
extern crate rodio;
extern crate discord_rich_presence;
extern crate dirs;

use clap::{App, Arg};
use lofty::{Accessor, Probe, AudioFile, TaggedFileExt};
use rodio::Sink;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

use std::sync::{Arc, Mutex};
use std::thread;

use std::process::{Command, exit};
use std::io::{self, Write, Read};
use std::path::Path;
use std::fs::File;
use std::fs;

use serde_derive::Deserialize;
use toml;

fn play_file(file_path: &str) -> Result<(), Box<dyn std::error::Error>> { // so begins the great refactoring
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

fn get_filepath_from_user() -> String {
    print!("Please enter the file path: ");
    io::stdout().flush().unwrap();

    let mut file_path = String::new();
    io::stdin().read_line(&mut file_path).expect("Failed to read path");

    file_path.trim().to_string()
}

// probably the worst code ever written ahead, i have no idea if this is even a good solution for this,
// as of writing it panics and doesn't open a new term, maybe just i3? i have no fucking clue. fuck this.

fn main() {

    // Specify the path to the TOML file
    let config_path = match dirs::config_dir() {
        Some(mut path) => {
            path.push("RustMusicCLI");
            path.push("config.toml");
            path
        }
        None => {
            panic!("Could not determine the configuration directory");
        }
    };

    // Open the file
    let mut file = match File::open(&config_path) {
        Ok(file) => file,
        Err(e) => panic!("Failed to open file: {}", e),
    };

    // what the fuck
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {
            // parse toml content to string
            let parsed_toml = match contents.parse::<toml::Value>() {
                Ok(toml) => toml,
                Err(e) => panic!("Failed to parse TOML: {}", e),
            };

            // access emulator value and open new terminal
            if let Some(config) = parsed_toml.get("config") {
                if let Some(emulator) = config.get("emulator") {
                    if let Some(emulator_str) = emulator.as_str() {
                        match Command::new(emulator_str).spawn() {
                            Ok(_) => println!("Successfully opened {}", emulator_str),
                            Err(e) => println!("Failed to open {}: {}", emulator_str, e),
                        }
                    } else {
                        println!("Emulator value is not a string");
                    }
                } else {
                    println!("Emulator key not found");
                }
            } else {
                println!("Config key not found");
            }
        }
        Err(e) => panic!("Failed to read file: {}", e),
    }
}
