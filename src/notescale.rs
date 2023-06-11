use std::sync::mpsc::{channel, Receiver, SendError, Sender};

use rodio::{self, Sink};

use rodio::OutputStreamHandle;

use crate::note::Note;

pub enum AudioControlFlow {
    Play { sink: Sink },
    Stop,
    Pause,
    Resume,
}

pub struct AudioPlayer {
    receiver: Receiver<AudioControlFlow>,
}
pub struct AudioPlayerInterface {
    sender: Sender<AudioControlFlow>,
}

pub fn create_audio_player() -> (AudioPlayer, AudioPlayerInterface) {
    let (sender, receiver) = channel();
    (AudioPlayer { receiver }, AudioPlayerInterface { sender })
}

impl AudioPlayer {
    pub fn lookup(&self) -> Result<(), rodio::PlayError> {
        let mut sink_tank: Vec<Sink> = Vec::new();

        for msg in self.receiver.iter() {
            sink_tank.retain(|sink| sink.len() > 0);

            match msg {
                AudioControlFlow::Play { sink } => sink_tank.push(sink),
                AudioControlFlow::Pause => sink_tank.iter().for_each(|sink| sink.pause()),
                AudioControlFlow::Resume => sink_tank.iter().for_each(|sink| sink.play()),
                AudioControlFlow::Stop => {
                    sink_tank.iter().for_each(|sink| sink.stop());
                    sink_tank = Vec::new()
                }
            }
        }
        Ok(())
    }
}

impl AudioPlayerInterface {
    fn get_sender(&self) -> Sender<AudioControlFlow> {
        self.sender.clone()
    }

    pub fn play(&self, sink: Sink) -> Result<(), SendError<AudioControlFlow>> {
        self.get_sender().send(AudioControlFlow::Play { sink })
    }

    pub fn pause(&self) -> Result<(), SendError<AudioControlFlow>> {
        self.get_sender().send(AudioControlFlow::Pause)
    }

    pub fn resume(&self) -> Result<(), SendError<AudioControlFlow>> {
        self.get_sender().send(AudioControlFlow::Resume)
    }

    pub fn stop(&self) -> Result<(), SendError<AudioControlFlow>> {
        self.get_sender().send(AudioControlFlow::Stop)
    }
}

pub trait NoteScale {
    fn play_note(
        &self,
        note: Note,
        player: &AudioPlayerInterface,
        output_stream_handle: &OutputStreamHandle,
    ) -> Result<(), rodio::PlayError>;
}
