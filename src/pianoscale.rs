use std::collections::HashMap;
use std::fs::File;

use log::info;

use strfmt::strfmt;

use awedio::sounds::decoders::WavDecoder;

use crate::note::Note;
use crate::notescale::NoteScale;
use crate::player::AudioPlayerInterface;

pub struct PianoScale {
    notes_data: HashMap<Note, File>,
}

impl PianoScale {
    pub fn from_files(template: &str) -> anyhow::Result<Self> {
        let mut notes_data = HashMap::new();
        info!("Load files");
        Note::LIST.into_iter().try_for_each(|note| {
            let file = strfmt(template, &HashMap::from([(String::from("note"), note)]))?;
            info!("Load {}", file);

            notes_data.insert(note, File::open(file)?);
            Ok::<(), anyhow::Error>(())
        })?;
        info!("Loaded {} files successfuly", notes_data.len());
        Ok(Self { notes_data })
    }
}

impl NoteScale for PianoScale {
    fn play_note(&self, note: Note, player: &AudioPlayerInterface) -> anyhow::Result<()> {
        if let Some(note_sound) = self.notes_data.get(&note) {
            player.play_sound(Box::new(WavDecoder::new(note_sound.try_clone()?)?))?;
            Ok(())
        } else {
            unreachable!("Shouldn't be reachable !")
        }
    }
}
