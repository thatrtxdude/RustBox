mod player;
mod discord;
mod ui;

use clap::{App, Arg};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line arguments
    let matches = App::new("RustCLIMusic")
        .arg(
            Arg::new("file")
                .required(true)
                .takes_value(true)
                .index(1)
                .help("Input file path to file that the player should play"),
        )
        .arg(Arg::new("repeat").long("repeat").help("Repeats song"))
        .get_matches();

    let file_path = matches.value_of("file").unwrap();
    let repeat = matches.is_present("repeat");

    player::play_music(file_path, repeat)
}
