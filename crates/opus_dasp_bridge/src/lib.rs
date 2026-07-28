use std::fmt::Debug;

use cpal::Host;

mod macros;

pub mod error;
pub mod input_stream;
pub mod output_stream;

pub struct AudioBridge {
    host: Host,
}

impl AudioBridge {
    pub fn new() -> Self {
        let host = cpal::default_host();

        Self { host }
    }
}

impl Debug for AudioBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioBridge").finish()
    }
}
