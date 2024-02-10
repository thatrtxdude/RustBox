extern crate clap;
extern crate discord_rich_presence;
extern crate metaflac;
extern crate rodio;

use clap::{App, Arg};
use metaflac::Tag;
use rodio::Sink;
use std::fs::File;
use std::io::BufReader;

use std::sync::{Arc, Mutex};
use std::thread;

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

fn main() {
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

    let file = File::open(file_path).unwrap();
    let mut buf_reader = BufReader::new(file);

    let tag = Tag::read_from(&mut buf_reader).unwrap();

    let title = tag
        .get_vorbis("title")
        .and_then(|v| v.map(|s| s.to_string()).next())
        .unwrap_or_else(|| String::from("Unknown Title"));

    let artist = tag
        .get_vorbis("artist")
        .and_then(|v| v.map(|s| s.to_string()).next())
        .unwrap_or_else(|| String::from("Unknown Artist"));

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
        return;
    };

    
    //discord rpc func
    let drp_result = DiscordIpcClient::new("1205851178731175976");

    if let Ok(mut drp) = drp_result {
        match drp.connect() {
            Ok(_) => {
                println!("Connected to Discord successfully.");

                let formatted_title = format!("Playing {} by {}", title, artist); //work around (not really), old method would drop temp val at end of statement

                let payload = activity::Activity::new().state(&formatted_title);

                match drp.set_activity(payload) {
                    Ok(_) => println!("Activity set successfully."),
                    Err(e) => println!("Failed to set activity: {}", e),
                }
            },
            Err(e) => println!("Failed to connect to Discord: {}", e),
        }
    } else {
        println!("Discord is not running. Continuing without Discord Rich Presence.");
    }

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
}
