use std::{
    sync::{
        Arc,
        atomic::{self, AtomicBool},
        mpsc::{Receiver, Sender, TryRecvError},
    },
    thread::{self, JoinHandle},
};

use crate::traits::SampleFormatConversion;

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
        input_sample_format: cpal::SampleFormat,
    ) -> Self {
        let process_audio_handle = thread::spawn(move || {
            Self::process_audio(
                input_channel,
                output_channel.clone(),
                volume,
                cutoff,
                exit,
                input_sample_format,
            )
        });

        Self {
            process_audio_handle,
        }
    }

    fn process_audio<T>(
        input_channel: Receiver<Vec<T>>,
        output_channel: Sender<Vec<f32>>,
        volume: Arc<atomic::AtomicU8>,
        cutoff: Arc<atomic::AtomicU8>,

        exit: Arc<AtomicBool>,
        input_sample_format: cpal::SampleFormat,
    ) where
        T: SampleFormatConversion<f32>,
    {
        loop {
            if exit.load(atomic::Ordering::Acquire) {
                break;
            };

            let buf: Vec<f32> = match input_channel.try_recv() {
                Ok(buf) => T::to_sample_buf(buf, Some(input_sample_format)).collect(),
                Err(e) => {
                    if let TryRecvError::Empty = e {
                        thread::yield_now();
                        continue;
                    } else {
                        log::warn!("Audio Processor: Input Channel sender closed: {}", e);
                        break;
                    };
                }
            };

            // Process audio
            log::trace!("Audio Processor: Processing sample {} bytes", buf.len() * 4);

            match output_channel.send(buf) {
                Ok(_) => {}
                Err(e) => {
                    log::warn!("Audio Prcessor: Output Channel receiver closed: {}", e);
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
