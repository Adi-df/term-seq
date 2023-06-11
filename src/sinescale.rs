use std::time::Duration;

use crate::note::Note;
use crate::notescale::NoteScale;
use crate::player::AudioPlayerInterface;

pub struct SineScale {
    freq: Box<dyn Fn(Note) -> f32>,
    duration: Duration,
    amplify: f32,
}

impl NoteScale for SineScale {
    fn play_note(&self, note: Note, player: &AudioPlayerInterface) -> anyhow::Result<()> {
        player.play_sound(Box::new(awedio::sounds::SineWav::new((self.freq)(note))))?;
        Ok(())
    }
}

impl SineScale {
    pub fn new(freq: Box<dyn Fn(Note) -> f32>, duration: Duration, amplify: f32) -> Self {
        Self {
            freq,
            duration,
            amplify,
        }
    }
}
