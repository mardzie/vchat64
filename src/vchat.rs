use std::{
    net::{SocketAddr, ToSocketAddrs},
    sync::{Arc, Mutex, RwLock, atomic::AtomicBool},
    thread::{self, JoinHandle},
};

use color_eyre::eyre::Result;
use crossbeam::channel::{Receiver, RecvTimeoutError, Sender};

use crate::{
    TIMEOUT,
    audio::Audio,
    helpers::should_exit,
    traits::SampleFormatConversion,
    types::{ArcMutex, ArcRwLock},
    udp_packet_net::MAX_PAYLOAD_SIZE,
    voice_net::{self, VoiceNet},
};

pub const AUDIO_CHANNELS_BUF_SIZE: usize = 1024 * 16;

pub struct VChat {
    audio: Audio,
    voice_net: ArcMutex<VoiceNet>,
    addresses: ArcRwLock<Vec<SocketAddr>>,

    udp_bridge_handle: JoinHandle<()>,
}

impl VChat {
    pub fn new<A>(addr: A, exit: Arc<AtomicBool>) -> Result<Self>
    where
        A: ToSocketAddrs,
    {
        let (mic_input_tx, mic_output_rx) = crossbeam::channel::bounded(AUDIO_CHANNELS_BUF_SIZE);
        let (speaker_input_tx, speaker_output_rx) =
            crossbeam::channel::bounded(AUDIO_CHANNELS_BUF_SIZE);

        let exit_c = exit.clone();
        let audio = Audio::new(mic_input_tx, speaker_output_rx, u8::MAX, 50, exit_c);

        let exit_c = exit.clone();
        let voice_net = Arc::new(Mutex::new(
            VoiceNet::new(addr, exit_c).expect("Failed to create VoiceNet."),
        ));

        let addresses = Arc::new(RwLock::new(Vec::with_capacity(8)));

        let voice_net_c = voice_net.clone();
        let addresses_c = addresses.clone();
        let exit_c = exit.clone();
        let output_sample_format = audio.output_sample_format();
        let udp_bridge_handle = thread::spawn(move || {
            Self::udp_bridge(
                voice_net_c,
                addresses_c,
                mic_output_rx,
                speaker_input_tx,
                output_sample_format,
                exit_c,
            )
        });

        audio.play();

        Ok(Self {
            audio,
            voice_net,
            addresses,

            udp_bridge_handle,
        })
    }

    fn udp_bridge(
        voice_net: ArcMutex<VoiceNet>,
        addresses: ArcRwLock<Vec<SocketAddr>>,

        input_rx: Receiver<Vec<f32>>,
        output_tx: Sender<Vec<f32>>,
        output_sample_format: cpal::SampleFormat,

        exit: Arc<AtomicBool>,
    ) {
        loop {
            if should_exit(&exit) {
                break;
            };

            if Self::input_udp_bridge(&voice_net, &input_rx, &addresses).is_err() {
                break;
            };

            if Self::udp_output_bridge(&voice_net, &output_tx, &output_sample_format).is_err() {
                break;
            };
        }

        log::warn!("UDP Bridge: Closed")
    }

    fn input_udp_bridge(
        voice_net: &ArcMutex<VoiceNet>,
        input_rx: &Receiver<Vec<f32>>,
        addresses: &ArcRwLock<Vec<SocketAddr>>,
    ) -> Result<(), ()> {
        let data = match input_rx.recv_timeout(TIMEOUT) {
            Ok(data) => data,
            Err(e) => {
                if let RecvTimeoutError::Timeout = e {
                    return Ok(());
                } else {
                    log::warn!("Input UDP Bridge: Input stream closed: {}", e);
                    return Err(());
                }
            }
        };

        // Convert into bytes and split up into packets.
        let byte_packets: Vec<Vec<u8>> = data
            .chunks(MAX_PAYLOAD_SIZE / 4)
            .map(|chunk| {
                chunk
                    .iter()
                    .flat_map(|sample| sample.to_be_bytes())
                    .collect::<Vec<u8>>()
            })
            .collect();

        log::trace!(
            "Input UDP Bridge: Preparing {} packet {} bytes.",
            byte_packets.len(),
            byte_packets[0].len()
        );

        let voice_net_lock = voice_net
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let addresses_lock = addresses
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for bytes in byte_packets {
            for addr in addresses_lock.iter() {
                match voice_net_lock.send(bytes.clone(), addr) {
                    Ok(_) => {}
                    Err(e) => match e {
                        voice_net::error::SendError::WouldBlock => {
                            log::warn!("Input UDP Bridge: Failed to send `Packet` to {}", addr);
                        }
                        voice_net::error::SendError::Io(e) => {
                            log::warn!(
                                "Inptu UDP Bridge: Failed to send `Packet` to {}: {}",
                                addr,
                                e
                            );
                        }
                    },
                };
            }
        }

        Ok(())
    }

    fn udp_output_bridge<T>(
        voice_net: &ArcMutex<VoiceNet>,
        output_tx: &crossbeam::channel::Sender<Vec<T>>,
        output_sample_format: &cpal::SampleFormat,
    ) -> Result<(), ()>
    where
        T: SampleFormatConversion<f32>,
    {
        const AUDIO_VALUE_BYTE_LEN: usize = 4;

        let (addr, bytes) = {
            let mut voice_net_lock = voice_net
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some((_, transmission)) = voice_net_lock.recv() {
                transmission
            } else {
                return Ok(());
            }
        };

        log::trace!(
            "UDP Output Bridge: Got message from {} with size {}",
            addr,
            bytes.len()
        );

        let data: Vec<f32> = bytes
            .chunks(AUDIO_VALUE_BYTE_LEN)
            .map(|chunk| {
                let mut buf = [0u8; AUDIO_VALUE_BYTE_LEN];
                buf.copy_from_slice(chunk);

                buf
            })
            .map(f32::from_be_bytes)
            .collect();

        let samples = T::from_sample_buf(data, Some(*output_sample_format)).collect();

        match output_tx.send(samples) {
            Ok(_) => {}
            Err(e) => {
                log::warn!("UDP Output Bridge: Output device closed channel: {}", e);
                return Err(());
            }
        };

        Ok(())
    }

    pub fn add_address(&self, addr: SocketAddr) {
        let mut addresses = self
            .addresses
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        addresses.push(addr);
        addresses.sort();
        addresses.dedup();
    }

    #[inline]
    pub fn get_addresses(&self) -> &std::sync::Arc<std::sync::RwLock<Vec<SocketAddr>>> {
        &self.addresses
    }

    pub fn remove_address(&self, addr: SocketAddr) {
        todo!()
    }

    #[inline]
    pub fn clear_addresses(&self) {
        self.addresses
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }

    #[inline]
    pub fn audio(&self) -> &Audio {
        &self.audio
    }

    #[inline]
    pub fn voice_net(&self) -> &ArcMutex<VoiceNet> {
        &self.voice_net
    }

    pub fn stop(self) {
        self.audio.stop();

        let _ = self.udp_bridge_handle.join();
    }
}
