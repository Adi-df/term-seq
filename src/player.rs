use std::fmt::Display;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use log::info;

use kira::manager::backend::DefaultBackend;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};
use kira::sound::streaming::StreamingSoundHandle;
use kira::sound::PlaybackState;
use kira::tween::Tween;

pub enum AudioControlFlow {
    PlayStatic { sound: Box<StaticSoundData> },
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
                AudioControlFlow::PlayStatic { .. } => "PlayStatic",
                AudioControlFlow::Stop => "Stop",
                AudioControlFlow::Pause => "Pause",
                AudioControlFlow::Resume => "Resume",
            }
        )
    }
}

pub struct AudioPlayer {
    manager: AudioManager,
    statics: Vec<StaticSoundHandle>,
    streams: Vec<StreamingSoundHandle<anyhow::Error>>,
    receiver: Receiver<AudioControlFlow>,
}
pub struct AudioPlayerInterface {
    sender: Sender<AudioControlFlow>,
}

pub fn create_audio_player() -> anyhow::Result<(AudioPlayer, AudioPlayerInterface)> {
    let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
    let (sender, receiver) = channel();
    Ok((
        AudioPlayer {
            receiver,
            manager,
            statics: Vec::new(),
            streams: Vec::new(),
        },
        AudioPlayerInterface { sender },
    ))
}

impl AudioPlayer {
    pub fn spawn_deamon(mut self) {
        thread::spawn(move || {
            self.lookup().unwrap();
        });
    }

    pub fn lookup(&mut self) -> anyhow::Result<()> {
        for msg in self.receiver.iter() {
            info!("Received control flow : {}", msg);
            info!("{} sounds playing", self.statics.len());

            self.statics
                .retain(|sound| matches!(sound.state(), PlaybackState::Stopped));

            match msg {
                AudioControlFlow::PlayStatic { sound } => {
                    let handler = self.manager.play(*sound)?;
                    self.statics.push(handler);
                }
                AudioControlFlow::Pause => {
                    self.manager.pause(Tween::default())?;
                }
                AudioControlFlow::Resume => {
                    self.manager.resume(Tween::default())?;
                }
                AudioControlFlow::Stop => {
                    self.manager.pause(Tween::default())?;
                    self.statics
                        .iter_mut()
                        .try_for_each(|sound| sound.stop(Tween::default()))?;
                    self.streams
                        .iter_mut()
                        .try_for_each(|sound| sound.stop(Tween::default()))?;

                    self.statics = Vec::new();
                    self.streams = Vec::new();
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

    pub fn play_sound(&self, sound: StaticSoundData) -> anyhow::Result<()> {
        Ok(self.get_sender().send(AudioControlFlow::PlayStatic {
            sound: Box::new(sound),
        })?)
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
