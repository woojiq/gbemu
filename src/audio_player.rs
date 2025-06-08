pub trait AudioPlayer: Send {
    fn play(&mut self, buff: crate::AudioBuff);
}

pub struct VoidAudioPlayer {}

impl VoidAudioPlayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl AudioPlayer for VoidAudioPlayer {
    fn play(&mut self, _buff: crate::AudioBuff) {}
}

pub struct CpalAudioPlayer {
    sender: std::sync::mpsc::Sender<crate::AudioBuff>,
}

impl CpalAudioPlayer {
    pub fn new(sender: std::sync::mpsc::Sender<crate::AudioBuff>) -> Self {
        Self { sender }
    }
}

impl AudioPlayer for CpalAudioPlayer {
    fn play(&mut self, buff: crate::AudioBuff) {
        let _ = self.sender.send((buff.0, buff.1));
    }
}
