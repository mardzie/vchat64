use std::{
    net::{SocketAddr, ToSocketAddrs},
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, RecvTimeoutError, Sender},
    },
    thread::{self, JoinHandle},
};

use color_eyre::eyre::Result;

use crate::{
    TIMEOUT,
    audio::Audio,
    traits::SampleFormatConversion,
    udp_packet_net::{MAX_PAYLOAD_SIZE, packet::Packet},
    voice_net::VoiceNet,
};

pub struct VChat {
    audio: Audio,
    voice_net: Arc<Mutex<VoiceNet>>,
    addresses: Arc<RwLock<Vec<SocketAddr>>>,

    udp_bridge_handle: JoinHandle<()>,
}

impl VChat {
    pub fn new<A>(addr: A, exit: Arc<AtomicBool>) -> Result<Self>
    where
        A: ToSocketAddrs,
    {
        let (mic_input_tx, mic_output_rx) = std::sync::mpsc::channel();
        let (speaker_input_tx, speaker_output_rx) = crossbeam::channel::unbounded();

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
        let udp_bridge_handle = thread::spawn(move || {
            Self::udp_bridge(
                voice_net_c,
                addresses_c,
                mic_output_rx,
                speaker_input_tx,
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
        voice_net: Arc<Mutex<VoiceNet>>,
        addresses: Arc<RwLock<Vec<SocketAddr>>>,

        input_rx: Receiver<Vec<f32>>,
        output_tx: crossbeam::channel::Sender<Vec<f32>>,
        output_sample_format: cpal::SampleFormat,

        exit: Arc<AtomicBool>,
    ) {
        loop {
            if exit.load(Ordering::Acquire) {
                break;
            };

            if let Err(_) = Self::input_udp_bridge(&voice_net, &input_rx) {
                break;
            };

            if let Err(_) = Self::udp_output_bridge(&voice_net, &output_tx, &output_sample_format) {
                break;
            };
        }

        log::warn!("UDP Bridge: Closed")
    }

    fn input_udp_bridge(
        voice_net: &Arc<Mutex<VoiceNet>>,
        input_rx: &Receiver<Vec<f32>>,
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

        for bytes in byte_packets {
            match udp_sender.send(Packet::from(bytes)) {
                Ok(_) => {}
                Err(_) => {
                    log::warn!("Input UDP Bridge: UDP Stream closed.");
                    return Err(());
                }
            };
        }

        Ok(())
    }

    fn udp_output_bridge<T>(
        voice_net: &Arc<Mutex<VoiceNet>>,
        output_tx: &crossbeam::channel::Sender<Vec<T>>,
        output_sample_format: &cpal::SampleFormat,
    ) -> Result<(), ()>
    where
        T: SampleFormatConversion<f32>,
    {
        const AUDIO_VALUE_BYTE_LEN: usize = 4;

        let (addr, bytes) = match udp_receiver.recv_timeout(TIMEOUT) {
            Ok(packet) => packet,
            Err(e) => {
                if let RecvTimeoutError::Timeout = e {
                    return Ok(());
                } else {
                    log::warn!("UDP Output Bridge: Failed to recv new packet: {}", e);
                    return Err(());
                }
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

        let samples = T::from_sample_buf(data, Some(output_sample_format)).collect();

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
    pub fn voice_net(&self) -> &VoiceNet {
        &self.voice_net
    }

    pub fn stop(self) {
        self.audio.stop();

        let _ = self.udp_bridge_handle.join();
    }
}
