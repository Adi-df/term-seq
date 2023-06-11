use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::note::{Note, UnknownNote};
use crate::notescale::NoteScale;
use crate::player::AudioPlayerInterface;

pub struct Track {
    pub content: Vec<Note>,
    pub current: usize,
    pub last_beat: Instant,
    note_scale: Option<Rc<dyn NoteScale>>,
    tempo: f64,
}

impl From<&Track> for Vec<&str> {
    fn from(track: &Track) -> Self {
        track.content.iter().map(|x| (*x).into()).collect()
    }
}

impl TryFrom<&[&str]> for Track {
    type Error = UnknownNote;
    fn try_from(track: &[&str]) -> Result<Track, Self::Error> {
        Ok(Track {
            content: track
                .iter()
                .map(|x| (*x).try_into())
                .collect::<Result<_, _>>()?,
            note_scale: None,
            current: 0,
            last_beat: Instant::now(),
            tempo: 0.,
        })
    }
}

impl Track {
    pub fn length(&self) -> usize {
        self.content.len()
    }

    pub fn set_tempo(self, tempo: f64) -> Self {
        Self { tempo, ..self }
    }

    pub fn set_note_scale(self, note_scale: Option<Rc<dyn NoteScale>>) -> Self {
        Self { note_scale, ..self }
    }

    pub fn should_beat(&self, beat: Duration) -> bool {
        self.last_beat.elapsed() > beat.mul_f64(1. / self.tempo)
    }

    pub fn beat(&mut self, player: &AudioPlayerInterface) -> anyhow::Result<()> {
        self.last_beat = Instant::now();
        self.current += 1;
        self.current %= self.length();

        if let Some(scale) = &self.note_scale {
            scale.play_note(self.content[self.current], player)?;
        }

        Ok(())
    }

    pub fn restart(&mut self) {
        self.current = 0;
    }
}
