use rodio;

use rodio::OutputStreamHandle;

use super::Note;

pub(crate) trait NoteScale {
    fn play_note(
        &self,
        note: Note,
        handle: &mut OutputStreamHandle,
    ) -> Result<(), rodio::PlayError>;
}
