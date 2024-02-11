# Work in progress CLI Music player written in Rust

Fast and lightweight CLI-Based music player written in Rust.

I started this project as a way to learn Rust. Code might not be the cleanest due to that.

Right now this project is in its infancy phase. The current focus is getting basic features to work before moving onto more QoL-stuff like a CLI-Based UI.

# Functionality

- [x] Basic playback features (Play, Pause, Repeat)
- [x] Discord RPC Status
- [x] Show bitrate
- [x] Show duration
- [ ] Progress bar 
- [ ] Playlists
- [ ] Display synced Lyrics (Maybe)
- [ ] Show remaining track time
- [ ] Show cover art and more details on Discord

# File Format support

| File Format 	| Support                  	| Added 	|
|:-----------:	|--------------------------	|-------	|
| FLAC        	| <ul><li>- [x] </li></ul> 	| [fbcded2](https://github.com/thatrtxdude/RustMusicCLI/commit/fbcded2bcd255a475414b296ff4e77911d27fc58) 	|
| MP3         	| <ul><li>- [x] </li></ul> 	| [f3fe90](https://github.com/thatrtxdude/RustMusicCLI/commit/f3fe90f0fdf36dd9e3479a0c87371b2313aeff10) |
| OGG         	| <ul><li>- [x] </li></ul> 	| [f3fe90](https://github.com/thatrtxdude/RustMusicCLI/commit/f3fe90f0fdf36dd9e3479a0c87371b2313aeff10) |
| WAV         	| <ul><li>- [ ] </li></ul> 	|     Working on it |
| AAC         	| <ul><li>- [ ] </li></ul> 	|     ???  	|


The file formats that aren't checked will be supported at a later date.

# Requirements

ALSA + PulseAudio are highly recommended.
Other audio drivers have not been tested and I can't gurantee functionality with them.

# Goal

The goal of this project is to create an easy to use, well-documented and fast music player with modern functionality such as playlists, shuffling, etc.

Integration with applications like Spotify are likely to never happen, as I like to actually **own** my music.

# License

This project is licensed under the GNU General Public License v3.0 - see the LICENSE file for details.
