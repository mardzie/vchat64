use crate::error::{PlayPauseError, StreamBuildError};

pub trait StreamControls {
    fn play(&mut self) -> Result<(), PlayPauseError>;
    fn pause(&mut self) -> Result<(), PlayPauseError>;
    fn playing(&self) -> bool;
}

pub trait BuildStreamTrait {
    fn build_stream<T, D, E>(
        &mut self,
        data_callback: D,
        error_callback: E,
    ) -> Result<(), StreamBuildError>
    where
        T: cpal::SizedSample,
        D: FnMut(&[T], &cpal::InputCallbackInfo) + Send + 'static,
        E: FnMut(cpal::Error) + Send + 'static;
}
