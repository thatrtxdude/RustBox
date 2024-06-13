use crossterm::event::{self, Event, KeyCode};
use rodio::Sink;
use std::sync::{Arc, Mutex};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

pub fn run_ui(
    sink: Arc<Mutex<Sink>>,
    bitrate: Option<u32>,
    overall_bitrate: Option<u32>,
    display_duration: String,
) {
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    loop {
        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(f.size());

                let controls = Paragraph::new("Controls: play, pause, bitrate, duration, quit")
                    .style(Style::default().fg(Color::White))
                    .block(Block::default().borders(Borders::ALL).title("Controls"));

                let info = Paragraph::new(format!(
                    "Audio Bitrate: {}\nOverall Bitrate: {}\nDuration: {}",
                    bitrate.unwrap_or(0),
                    overall_bitrate.unwrap_or(0),
                    display_duration
                ))
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::ALL).title("Info"));

                f.render_widget(controls, chunks[0]);
                f.render_widget(info, chunks[1]);
            })
            .unwrap();

        if event::poll(std::time::Duration::from_millis(100)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('p') => sink.lock().unwrap().play(),
                    KeyCode::Char('s') => sink.lock().unwrap().pause(),
                    KeyCode::Char('b') => println!(
                        "Audio Bitrate: {}, Overall Bitrate: {}",
                        bitrate.unwrap_or(0),
                        overall_bitrate.unwrap_or(0)
                    ),
                    KeyCode::Char('d') => println!("{}", display_duration),
                    _ => {}
                }
            }
        }
    }
}
