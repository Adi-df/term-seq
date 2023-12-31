use std::error::Error;
use std::fmt::Display;

use strfmt::DisplayStr;

#[derive(Clone, Debug)]
pub struct UnknownNote(String);

impl Display for UnknownNote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The provided string '{}' is unknown", self.0)
    }
}

impl Error for UnknownNote {}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Note {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

impl Note {
    pub const LIST: [Note; 7] = [
        Self::C,
        Self::D,
        Self::E,
        Self::F,
        Self::G,
        Self::A,
        Self::B,
    ];
}

impl From<Note> for &str {
    fn from(note: Note) -> Self {
        match note {
            Note::C => "C",
            Note::D => "D",
            Note::E => "E",
            Note::F => "F",
            Note::G => "G",
            Note::A => "A",
            Note::B => "B",
        }
    }
}

impl TryFrom<&str> for Note {
    type Error = UnknownNote;
    fn try_from(note: &str) -> Result<Self, Self::Error> {
        match note {
            "C" => Ok(Note::C),
            "D" => Ok(Note::D),
            "E" => Ok(Note::E),
            "F" => Ok(Note::F),
            "G" => Ok(Note::G),
            "A" => Ok(Note::A),
            "B" => Ok(Note::B),
            _ => Err(UnknownNote(note.to_string())),
        }
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(<&str>::from(*self))
    }
}

impl DisplayStr for Note {
    fn display_str(&self, f: &mut strfmt::Formatter) -> strfmt::Result<()> {
        f.str(<&str>::from(*self))
    }
}
