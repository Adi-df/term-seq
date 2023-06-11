use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Cursor, Read};

use rodio::{Decoder, OutputStreamHandle, Sink, Source};

use strfmt::strfmt;

use crate::note::Note;
use crate::notescale::AudioPlayerInterface;
use crate::notescale::NoteScale;

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
    notes_data: HashMap<Note, RawSound>,
}

impl PianoScale {
    pub fn from_files(template: &str) -> io::Result<PianoScale> {
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
        Ok(Self { notes_data })
    }
}

impl NoteScale for PianoScale {
    fn play_note(
        &self,
        note: Note,
        player: &AudioPlayerInterface,
        output_stream_handle: &OutputStreamHandle,
    ) -> Result<(), rodio::PlayError> {
        if let Some(note_sound) = self.notes_data.get(&note) {
            let sink = Sink::try_new(output_stream_handle)?;
            sink.append(note_sound.decoder().convert_samples::<f32>());

            player.play(sink).unwrap();
            Ok(())
        } else {
            unreachable!("Shouldn't be reachable !")
        }
    }
}
