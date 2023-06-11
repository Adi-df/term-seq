use rodio::OutputStreamHandle;

use crate::note::Note;
use crate::player::AudioPlayerInterface;

pub trait NoteScale {
    fn play_note(
        &self,
        note: Note,
        player: &AudioPlayerInterface,
        output_stream_handle: &OutputStreamHandle,
    ) -> Result<(), rodio::PlayError>;
}
