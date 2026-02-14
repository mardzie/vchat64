use std::{
    sync::{
        Arc,
        atomic::{self, AtomicBool},
        mpsc::{Receiver, Sender, TryRecvError},
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

        exit: Arc<AtomicBool>,
    ) -> Self {
        let process_audio_handle = thread::spawn(move || {
            Self::process_audio(input_channel, output_channel.clone(), volume, cutoff, exit)
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

        exit: Arc<AtomicBool>,
    ) {
        loop {
            if exit.load(atomic::Ordering::Acquire) {
                break;
            };

            let buf = match input_channel.try_recv() {
                Ok(buf) => buf,
                Err(e) => {
                    if let TryRecvError::Empty = e {
                        thread::yield_now();
                        continue;
                    } else {
                        log::warn!("Process Audio: Input Channel sender closed: {}", e);
                        break;
                    };
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

        log::info!("Audio Processer: Stopped.");
    }

    pub fn stop(self) {
        let _ = self.process_audio_handle.join();
    }
}
