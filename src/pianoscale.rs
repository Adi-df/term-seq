use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Cursor, Read};
use std::sync::Arc;

use log::info;
use rodio::{Decoder, OutputStreamHandle, Sink, Source};

use strfmt::strfmt;

use crate::note::Note;
use crate::notescale::NoteScale;
use crate::player::AudioPlayerInterface;

struct RawSound {
    data: Vec<u8>,
}

impl AsRef<[u8]> for RawSound {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl RawSound {
    fn load_from_file(filepath: &str) -> io::Result<Self> {
        let mut buf = Vec::new();
        let mut file = File::open(filepath)?;
        file.read_to_end(&mut buf)?;
        Ok(Self { data: buf })
    }

    fn cursor(&self) -> Cursor<Self> {
        Cursor::new(Self {
            data: self.data.clone(),
        })
    }

    fn decoder(&self) -> Decoder<Cursor<Self>> {
        Decoder::new(self.cursor()).unwrap()
    }
}

pub struct PianoScale {
    stream: Arc<Sink>,
    notes_data: HashMap<Note, RawSound>,
}

impl PianoScale {
    pub fn from_files(
        template: &str,
        player: &AudioPlayerInterface,
        output_stream: &OutputStreamHandle,
    ) -> io::Result<PianoScale> {
        let stream = Arc::new(Sink::try_new(output_stream).unwrap());
        player.register_stream(stream.clone()).unwrap();

        let mut notes_data = HashMap::new();
        Note::list().into_iter().try_for_each(|note| {
            println!(
                "{}",
                &strfmt(template, &HashMap::from([(String::from("note"), note)])).unwrap()
            );
            notes_data.insert(
                note,
                RawSound::load_from_file(
                    &strfmt(template, &HashMap::from([(String::from("note"), note)])).unwrap(),
                )?,
            );
            Ok::<(), io::Error>(())
        })?;
        Ok(Self { stream, notes_data })
    }
}

impl NoteScale for PianoScale {
    fn play_note(
        &self,
        note: Note,
        _player: &AudioPlayerInterface,
        _output_stream_handle: &OutputStreamHandle,
    ) -> Result<(), rodio::PlayError> {
        if let Some(note_sound) = self.notes_data.get(&note) {
            info!("Stream len before : {}", self.stream.len());

            self.stream.clear();
            self.stream
                .append(note_sound.decoder().convert_samples::<f32>());

            info!("Stream len after : {}", self.stream.len());
            Ok(())
        } else {
            unreachable!("Shouldn't be reachable !")
        }
    }
}
