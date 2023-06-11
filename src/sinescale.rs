use std::time::Duration;

use rodio::source::SineWave;
use rodio::{OutputStreamHandle, Sink, Source};

use crate::note::Note;
use crate::notescale::AudioPlayerInterface;
use crate::notescale::NoteScale;

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
    ) -> Result<(), rodio::PlayError> {
        let sink = Sink::try_new(output_stream_hanle)?;
        sink.append(
            SineWave::new((self.freq)(note))
                .take_duration(self.duration)
                .amplify(self.amplify),
        );

        player.play(sink).unwrap();
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
