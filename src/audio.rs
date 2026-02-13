use cpal::{Host, default_host};

use crate::audio::{input::InputStream, output::OutputStream};

pub mod config_filter;
pub mod error;
pub mod input;
pub mod output;

pub struct Audio {
    host: Host,
    input: InputStream,
    output: OutputStream,
}

impl Audio {
    pub fn new<T, ID, IE, OD, OE>(
        input_data_callback: ID,
        input_error_callback: IE,
        output_data_callback: OD,
        output_error_callback: OE,
    ) -> Self
    where
        T: cpal::SizedSample,
        ID: FnMut(&[T], &cpal::InputCallbackInfo) + Send + 'static,
        IE: FnMut(cpal::StreamError) + Send + 'static,
        OD: FnMut(&mut [T], &cpal::OutputCallbackInfo) + Send + 'static,
        OE: FnMut(cpal::StreamError) + Send + 'static,
    {
        let host = default_host();

        Self {
            input: InputStream::new(&host, input_data_callback, input_error_callback)
                .expect("Failed to create new input stream."),
            output: OutputStream::new(&host, output_data_callback, output_error_callback)
                .expect("Failed to create new output stream."),
            host,
        }
    }

    pub fn play(&self) {
        self.play_input();
        self.play_output();
    }

    pub fn play_input(&self) {
        self.input.play().expect("Failed to play input stream.");
    }

    pub fn play_output(&self) {
        self.output.play().expect("Failed to play output stream.");
    }

    pub fn pause(&self) {
        self.pause_input();
        self.pause_output();
    }

    pub fn pause_input(&self) {
        self.input.pause().expect("Failed to pause input stream.");
    }

    pub fn pause_output(&self) {
        self.output.pause().expect("Failed to pause output stream.");
    }
}
