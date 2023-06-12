use std::collections::HashMap;
use std::f64::consts::PI;
use std::sync::Arc;
use std::time::Duration;

use kira::dsp::Frame;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};

use crate::note::Note;
use crate::notescale::NoteScale;
use crate::player::AudioPlayerInterface;

pub struct SineScale {
    freqs: HashMap<Note, f32>,
    duration: Duration,
}

impl NoteScale for SineScale {
    fn play_note(&self, note: Note, player: &AudioPlayerInterface) -> anyhow::Result<()> {
        if let Some(freq) = self.freqs.get(&note) {
            player.play_sound(StaticSoundData {
                sample_rate: Self::SAMPLE_NUMBER,
                frames: self.generate_wave(*freq),
                settings: StaticSoundSettings::default(),
            })?;
        } else {
            unreachable!("Shouldn't be reached");
        }
        Ok(())
    }
}

impl SineScale {
    const SAMPLE_NUMBER: u32 = 48000;

    pub fn new(freqs: HashMap<Note, f32>, duration: Duration) -> Self {
        Self { freqs, duration }
    }

    fn generate_wave(&self, freq: f32) -> Arc<[Frame]> {
        (0..(f64::from(Self::SAMPLE_NUMBER) * self.duration.as_secs_f64()).round() as u32)
            .map(|s| {
                Frame::from_mono(
                    (2.0 * PI * f64::from(freq) * f64::from(s) / f64::from(Self::SAMPLE_NUMBER))
                        .sin() as f32
                        * 0.025,
                )
            })
            .collect()
    }
}
