use std::collections::HashMap;
use std::fs::File;

use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use log::info;

use strfmt::strfmt;

use crate::note::Note;
use crate::notescale::NoteScale;
use crate::player::AudioPlayerInterface;

pub struct FileScale {
    notes_data: HashMap<Note, StaticSoundData>,
}

impl FileScale {
    pub fn from_files(template: &str) -> anyhow::Result<Self> {
        let mut notes_data = HashMap::new();

        info!("Load files");

        Note::LIST.into_iter().try_for_each(|note| {
            let file_name = strfmt(template, &HashMap::from([(String::from("note"), note)]))?;

            info!("Load {}", file_name);

            let file = File::open(file_name)?;

            notes_data.insert(
                note,
                StaticSoundData::from_media_source(file, StaticSoundSettings::default())?,
            );
            Ok::<(), anyhow::Error>(())
        })?;

        info!("Loaded {} files successfuly", notes_data.len());

        Ok(Self { notes_data })
    }
}

impl NoteScale for FileScale {
    fn play_note(&self, note: Note, player: &AudioPlayerInterface) -> anyhow::Result<()> {
        if let Some(note_sound) = self.notes_data.get(&note) {
            player.play_sound(note_sound.clone())?;
            Ok(())
        } else {
            unreachable!("Shouldn't be reachable !")
        }
    }
}
