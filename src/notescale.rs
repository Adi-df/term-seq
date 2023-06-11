use crate::note::Note;
use crate::player::AudioPlayerInterface;

pub trait NoteScale {
    fn play_note(&self, note: Note, player: &AudioPlayerInterface) -> anyhow::Result<()>;
}
