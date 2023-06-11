use std::sync::mpsc::{channel, Receiver, SendError, Sender};
use std::sync::Arc;

use log::info;

use rodio::Sink;

pub enum AudioControlFlow {
    PlaySink { sink: Sink },
    RegisterStream { stream: Arc<Sink> },
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
    pub fn lookup(&self) {
        let mut sink_tank: Vec<Sink> = Vec::new();
        let mut stream_tank: Vec<Arc<Sink>> = Vec::new();

        for msg in self.receiver.iter() {
            info!("Received control flow");
            info!("{} sinks playings", sink_tank.len());
            info!("{} stream playings", stream_tank.len());

            sink_tank.retain(|sink| !sink.empty());

            match msg {
                AudioControlFlow::PlaySink { sink } => sink_tank.push(sink),
                AudioControlFlow::RegisterStream { stream } => stream_tank.push(stream),
                AudioControlFlow::Pause => {
                    sink_tank.iter().for_each(Sink::pause);
                    stream_tank.iter().for_each(|stream| stream.pause());
                }
                AudioControlFlow::Resume => {
                    sink_tank.iter().for_each(Sink::play);
                    stream_tank.iter().for_each(|stream| stream.play());
                }
                AudioControlFlow::Stop => {
                    stream_tank.iter().for_each(|stream| stream.stop());
                    sink_tank.iter().for_each(Sink::stop);
                    sink_tank = Vec::new();
                }
            }
        }
    }
}

impl AudioPlayerInterface {
    fn get_sender(&self) -> Sender<AudioControlFlow> {
        self.sender.clone()
    }

    pub fn play_sink(&self, sink: Sink) -> Result<(), SendError<AudioControlFlow>> {
        self.get_sender().send(AudioControlFlow::PlaySink { sink })
    }

    pub fn register_stream(&self, stream: Arc<Sink>) -> Result<(), SendError<AudioControlFlow>> {
        self.get_sender()
            .send(AudioControlFlow::RegisterStream { stream })
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
