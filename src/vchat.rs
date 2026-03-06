use std::{
    net::{SocketAddr, ToSocketAddrs},
    sync::{
        Arc,
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
    udp_packet_net::{MAX_PAYLOAD_SIZE, UdpPacketNet, packet::Packet},
};

pub struct VChat {
    audio: Audio,
    udp_net: UdpPacketNet,

    input_udp_bridge_handle: JoinHandle<()>,
    udp_output_bridge_handle: JoinHandle<()>,
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
        let (udp_net, udp_sender, udp_receiver) = UdpPacketNet::new(addr, exit_c)?;

        let exit_c = exit.clone();
        let input_udp_bridge_handle =
            thread::spawn(move || Self::input_udp_bridge(mic_output_rx, udp_sender, exit_c));

        let output_sample_format = audio.output_sample_format();
        let udp_output_bridge_handle = thread::spawn(move || {
            Self::udp_output_bridge(udp_receiver, speaker_input_tx, exit, output_sample_format)
        });

        audio.play();

        Ok(Self {
            audio,
            udp_net,

            input_udp_bridge_handle,
            udp_output_bridge_handle,
        })
    }

    fn input_udp_bridge(
        input_rx: Receiver<Vec<f32>>,
        udp_sender: Sender<Packet>,
        exit: Arc<AtomicBool>,
    ) {
        loop {
            if exit.load(Ordering::Acquire) {
                break;
            };

            let data = match input_rx.recv_timeout(TIMEOUT) {
                Ok(data) => data,
                Err(e) => {
                    if let RecvTimeoutError::Timeout = e {
                        thread::yield_now();
                        continue;
                    } else {
                        log::warn!("Input UDP Bridge: Input stream closed: {}", e);
                        break;
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
                        break;
                    }
                };
            }
        }

        log::info!("Input UDP Bridge: Stopped.");
    }

    fn udp_output_bridge<T>(
        udp_receiver: Receiver<(SocketAddr, Vec<u8>)>,
        output_tx: crossbeam::channel::Sender<Vec<T>>,
        exit: Arc<AtomicBool>,
        output_sample_format: cpal::SampleFormat,
    ) where
        T: SampleFormatConversion<f32>,
    {
        const AUDIO_VALUE_BYTE_LEN: usize = 4;

        loop {
            if exit.load(Ordering::Acquire) {
                break;
            };

            let (addr, bytes) = match udp_receiver.recv_timeout(TIMEOUT) {
                Ok(packet) => packet,
                Err(e) => {
                    if let RecvTimeoutError::Timeout = e {
                        thread::yield_now();
                        continue;
                    } else {
                        log::warn!("UDP Output Bridge: Failed to recv new packet: {}", e);
                        break;
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
                    break;
                }
            };
        }

        log::info!("UDP Output Bridge: Stopped.");
    }

    pub fn add_address(&self, addr: SocketAddr) {
        let mut addresses = self
            .udp_net
            .addresses
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        addresses.push(addr);
        addresses.sort();
        addresses.dedup();
    }

    #[inline]
    pub fn get_addresses(&self) -> &std::sync::Arc<std::sync::RwLock<Vec<SocketAddr>>> {
        &self.udp_net.addresses
    }

    #[inline]
    pub fn clear_addresses(&self) {
        self.udp_net
            .addresses
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }

    #[inline]
    pub fn audio(&self) -> &Audio {
        &self.audio
    }

    #[inline]
    pub fn udp_net(&self) -> &UdpPacketNet {
        &self.udp_net
    }

    pub fn stop(self) {
        self.audio.stop();
        self.udp_net.stop();

        let _ = self.input_udp_bridge_handle.join();
        let _ = self.udp_output_bridge_handle.join();
    }
}
