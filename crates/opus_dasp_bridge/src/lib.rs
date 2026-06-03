use std::fmt::Debug;

use cpal::Host;

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
