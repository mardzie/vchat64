use std::{
    sync::{
        Arc, atomic,
        mpsc::{Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

#[derive(Debug)]
pub struct AudioProcessor {
    process_audio_handle: JoinHandle<()>,
}

impl AudioProcessor {
    pub fn new(
        input_channel: Receiver<Vec<f32>>,
        output_channel: Sender<Vec<f32>>,
        volume: Arc<atomic::AtomicU8>,
        cutoff: Arc<atomic::AtomicU8>,
    ) -> Self {
        let process_audio_handle = thread::spawn(move || {
            Self::process_audio(input_channel, output_channel.clone(), volume, cutoff)
        });

        Self {
            process_audio_handle,
        }
    }

    fn process_audio(
        input_channel: Receiver<Vec<f32>>,
        output_channel: Sender<Vec<f32>>,
        volume: Arc<atomic::AtomicU8>,
        cutoff: Arc<atomic::AtomicU8>,
    ) {
        loop {
            let buf = match input_channel.recv() {
                Ok(buf) => buf,
                Err(e) => {
                    log::warn!("Process Audio: Input Channel sender closed: {}", e);
                    break;
                }
            };
            
            // Process audio

            match output_channel.send(buf) {
                Ok(_) => {}
                Err(e) => {
                    log::warn!("Process Audio: Output Channel receiver closed: {}", e);
                    break;
                }
            };
        }
    }
}
