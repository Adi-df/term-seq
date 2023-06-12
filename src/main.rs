use std::collections::HashMap;
use std::io::{self, Stdout};
use std::iter::repeat;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::{panic, process};

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

mod filescale;
mod note;
mod notescale;
mod player;
mod sinescale;
mod track;

use crate::filescale::FileScale;
use crate::note::Note;
use crate::player::{create_audio_player, AudioPlayerInterface};
use crate::sinescale::SineScale;
use crate::track::Track;

type Term = Terminal<CrosstermBackend<Stdout>>;

struct AppData {
    tracks: Vec<Track>,
    player: AudioPlayerInterface,
    playing: bool,
    beat: Duration,
}

fn main() -> anyhow::Result<()> {
    log_to_file("logs.log", log::LevelFilter::Info)?;

    set_panic_hook();

    info!("Start");

    info!("Create audio player");
    let (player, interface) = create_audio_player()?;

    info!("Load scales");

    let sine_scale = Rc::new(SineScale::new(
        HashMap::from([
            (Note::C, 261.63),
            (Note::D, 293.66),
            (Note::E, 329.63),
            (Note::F, 349.23),
            (Note::G, 392.00),
            (Note::A, 440.00),
            (Note::B, 493.88),
        ]),
        Duration::from_millis(500),
    ));

    let piano_scale = Rc::new(FileScale::from_files(
        "assets/PianoPhase/N{note}_piano_phase.wav",
    )?);

    info!("Init app data");
    let mut app_data = AppData {
        tracks: vec![
            Track::try_from(&["E", "F", "B", "C", "D", "F", "E", "C", "B", "F", "D", "C"][..])?
                .set_tempo(4.)
                .set_note_scale(Some(piano_scale.clone())),
            Track::try_from(&["C", "D", "E"][..])?
                .set_tempo(1.)
                .set_note_scale(Some(sine_scale.clone())),
        ],
        player: interface,
        playing: false,
        beat: Duration::from_secs(1),
    };

    info!("Start audio player deamon");
    player.spawn_deamon();

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

    info!("Exit");

    res
}

fn set_panic_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        disable_raw_mode().unwrap();
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture).unwrap();

        default_hook(info);
        process::exit(1);
    }));
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
            app_data.tracks.iter_mut().try_for_each(|track| {
                if track.should_beat(app_data.beat) {
                    track.beat(&app_data.player)
                } else {
                    Ok(())
                }
            })?;
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
                        app_data.playing = false;
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
