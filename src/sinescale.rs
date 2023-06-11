use std::time::Duration;

use rodio::source::SineWave;
use rodio::{OutputStreamHandle, Sink, Source};

use crate::note::Note;
use crate::notescale::NoteScale;
use crate::player::AudioPlayerInterface;

pub struct SineScale {
    freq: Box<dyn Fn(Note) -> f32>,
    duration: Duration,
    amplify: f32,
}

impl NoteScale for SineScale {
    fn play_note(
        &self,
        note: Note,
        player: &AudioPlayerInterface,
        output_stream_hanle: &OutputStreamHandle,
    ) -> anyhow::Result<()> {
        let sink = Sink::try_new(output_stream_hanle)?;
        sink.append(
            SineWave::new((self.freq)(note))
                .take_duration(self.duration)
                .amplify(self.amplify),
        );

        player.play_sink(sink)?;
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
