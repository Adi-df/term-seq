use std::io::{self, Stdout};
use std::iter::repeat;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

use log::info;
use simple_logging::log_to_file;

use tui::backend::CrosstermBackend;
use tui::layout::Constraint;
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders, Cell, Row, Table};
use tui::Terminal;

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

use rodio::{OutputStream, OutputStreamHandle};

mod note;
mod notescale;
mod pianoscale;
mod player;
mod sinescale;
mod track;

use crate::note::Note;
use crate::pianoscale::PianoScale;
use crate::player::{create_audio_player, AudioPlayerInterface};
use crate::sinescale::SineScale;
use crate::track::Track;

type Term = Terminal<CrosstermBackend<Stdout>>;

struct AppData {
    tracks: Vec<Track>,
    player: AudioPlayerInterface,
    stream_handle: OutputStreamHandle,
    playing: bool,
    beat: Duration,
}

fn main() -> anyhow::Result<()> {
    log_to_file("logs.log", log::LevelFilter::Info)?;

    info!("Load scales");
    let (_stream, stream_handle) = OutputStream::try_default()?;

    let (player, interface) = create_audio_player();

    let sine_scale = Rc::new(SineScale::new(
        Box::new(|note| match note {
            Note::C => 261.63,
            Note::D => 293.66,
            Note::E => 329.63,
            Note::F => 349.23,
            Note::G => 392.00,
            Note::A => 440.00,
            Note::B => 493.88,
        }),
        Duration::from_millis(500),
        0.10,
    ));

    let piano_scale = Rc::new(PianoScale::from_files(
        "assets/GrandPiano/{note}4.wav",
        &interface,
        &stream_handle,
    )?);

    let mut app_data = AppData {
        tracks: vec![
            Track::try_from(&["C", "D", "E", "F", "G", "A", "B"][..])?
                .set_tempo(2.)
                .set_note_scale(Some(sine_scale.clone())),
            Track::try_from(&["C", "D", "E"][..])?.set_tempo(2.),
        ],
        player: interface,
        stream_handle,
        playing: false,
        beat: Duration::from_secs(1),
    };

    thread::spawn(move || player.lookup());

    info!("Start TUI");

    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = mainloop(&mut terminal, &mut app_data);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    info!("Stop");

    Ok(res?)
}

fn mainloop(terminal: &mut Term, app_data: &mut AppData) -> anyhow::Result<()> {
    let length = app_data.tracks.iter().map(Track::length).max().unwrap_or(0);
    let sizes = [Constraint::Length(8)]
        .into_iter()
        .chain(repeat(Constraint::Length(3)).take(length))
        .collect::<Vec<_>>();

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let tracks = Table::new(app_data.tracks.iter().enumerate().map(|(no, track)| {
                Row::new(
                    [Cell::from(Span::from(format!("Track {}", no + 1)))
                        .style(Style::default().bg(Color::Red))]
                    .into_iter()
                    .chain(track.content.iter().enumerate().map(|(i, note)| {
                        Cell::from(Span::styled(
                            <&str>::from(*note),
                            Style::default().fg(Color::Black),
                        ))
                        .style(Style::default().bg(
                            if i == track.current && app_data.playing {
                                Color::Red
                            } else {
                                Color::White
                            },
                        ))
                    }))
                    .collect::<Vec<_>>(),
                )
                .height(2)
                .bottom_margin(1)
            }))
            .block(
                Block::default()
                    .title("Term Sequencer")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(if app_data.playing {
                        Color::Green
                    } else {
                        Color::Red
                    })),
            )
            .style(Style::default().fg(Color::White))
            .widths(&sizes)
            .header(
                Row::new(
                    ["Track no".to_string()]
                        .into_iter()
                        .chain((1..=length).map(|x| x.to_string()))
                        .collect::<Vec<_>>(),
                )
                .style(Style::default().bg(Color::Green).fg(Color::Black))
                .bottom_margin(2),
            );

            f.render_widget(tracks, size);
        })?;

        if app_data.playing {
            app_data
                .tracks
                .iter_mut()
                .map(|track| {
                    if track.should_beat(app_data.beat) {
                        track.beat(&app_data.player, &app_data.stream_handle)
                    } else {
                        Ok(())
                    }
                })
                .collect::<anyhow::Result<()>>()?;
        }

        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Home | KeyCode::Esc | KeyCode::Char('q') => {
                        app_data.player.stop()?;
                        break;
                    }
                    KeyCode::Char('c') if k.modifiers == KeyModifiers::CONTROL => {
                        app_data.player.stop()?;
                        break;
                    }
                    KeyCode::Char('r') => {
                        app_data.tracks.iter_mut().for_each(Track::restart);
                        app_data.player.stop()?;
                    }
                    KeyCode::Char(' ') => {
                        if app_data.playing {
                            app_data.player.pause()?;
                        } else {
                            app_data.player.resume()?;
                        }

                        app_data
                            .tracks
                            .iter_mut()
                            .for_each(|track| track.last_beat = Instant::now());
                        app_data.playing = !app_data.playing;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
