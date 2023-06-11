use std::fmt::Display;
use std::sync::mpsc::{channel, Receiver, Sender};

use log::info;

use awedio::manager::Manager;
use awedio::Sound;

pub enum AudioControlFlow {
    Play { sound: Box<dyn Sound + Send + Sync> },
    Stop,
    Pause,
    Resume,
}

impl Display for AudioControlFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AudioControlFlow::Play { .. } => "Play",
                AudioControlFlow::Stop => "Stop",
                AudioControlFlow::Pause => "Pause",
                AudioControlFlow::Resume => "Resume",
            }
        )
    }
}

pub struct AudioPlayer {
    manager: Manager,
    receiver: Receiver<AudioControlFlow>,
}
pub struct AudioPlayerInterface {
    sender: Sender<AudioControlFlow>,
}

pub fn create_audio_player() -> anyhow::Result<(AudioPlayer, AudioPlayerInterface)> {
    let (manager, _backend) = awedio::start()?;
    let (sender, receiver) = channel();
    Ok((
        AudioPlayer { receiver, manager },
        AudioPlayerInterface { sender },
    ))
}

impl AudioPlayer {
    pub fn lookup(&mut self) {
        for msg in self.receiver.iter() {
            info!("Received control flow : {}", msg);

            match msg {
                AudioControlFlow::Play { sound } => self.manager.play(sound),
                AudioControlFlow::Pause => {
                    // unimplemented!();
                }
                AudioControlFlow::Resume => {
                    // self.manager.play()
                }
                AudioControlFlow::Stop => {
                    self.manager.clear();
                }
            }
        }
    }
}

impl AudioPlayerInterface {
    fn get_sender(&self) -> Sender<AudioControlFlow> {
        self.sender.clone()
    }

    pub fn play_sound(&self, sound: Box<dyn Sound + Send + Sync>) -> anyhow::Result<()> {
        Ok(self.get_sender().send(AudioControlFlow::Play { sound })?)
    }

    pub fn pause(&self) -> anyhow::Result<()> {
        Ok(self.get_sender().send(AudioControlFlow::Pause)?)
    }

    pub fn resume(&self) -> anyhow::Result<()> {
        Ok(self.get_sender().send(AudioControlFlow::Resume)?)
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        Ok(self.get_sender().send(AudioControlFlow::Stop)?)
    }
}
