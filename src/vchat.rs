use std::{
    net::{SocketAddr, ToSocketAddrs},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, RecvTimeoutError, Sender},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use color_eyre::eyre::Result;
use rayon::{
    iter::{
        IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
    },
    slice::ParallelSliceMut,
};

use crate::{
    TIMEOUT,
    audio::Audio,
    udp_net::{UdpNet, packet::Packet},
};

pub struct VChat {
    audio: Audio,
    udp_net: UdpNet,

    input_udp_bridge_handle: JoinHandle<()>,
    udp_output_bridge_handle: JoinHandle<()>,
}

impl VChat {
    pub fn new<A>(addr: A, exit: Arc<AtomicBool>) -> Result<Self>
    where
        A: ToSocketAddrs,
    {
        let (mic_input_tx, mic_output_rx) = std::sync::mpsc::channel();
        let (speaker_input_tx, speaker_output_rx) = std::sync::mpsc::channel();

        let (udp_net, udp_sender, udp_receiver) = UdpNet::new(addr)?;

        let exit_c = exit.clone();
        let input_udp_bridge_handle =
            thread::spawn(move || Self::input_udp_bridge(mic_output_rx, udp_sender, exit_c));

        let udp_output_bridge_handle =
            thread::spawn(move || Self::udp_output_bridge(udp_receiver, speaker_input_tx, exit));

        let audio = Audio::new(mic_input_tx, speaker_output_rx, u8::MAX, 50);
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
                        continue;
                    } else {
                        log::warn!("Input UDP Bridge: Input stream closed: {}", e);
                        break;
                    }
                }
            };

            let bytes: Vec<u8> = data.par_iter().map(|x| x.to_be_bytes()).flatten().collect();

            match udp_sender.send(Packet::from(bytes)) {
                Ok(_) => {}
                Err(_) => {
                    log::warn!("Input UDP Bridge: UDP Stream closed.");
                    break;
                }
            };
        }

        log::info!("Input UDP Bridge: Stopped.");
    }

    fn udp_output_bridge(
        udp_receiver: Receiver<(SocketAddr, Vec<u8>)>,
        output_tx: Sender<Vec<f32>>,
        exit: Arc<AtomicBool>,
    ) {
        const AUDIO_VALUE_BYTE_LEN: usize = 4;

        loop {
            if exit.load(Ordering::Acquire) {
                break;
            };

            let (addr, bytes) = match udp_receiver.recv_timeout(TIMEOUT) {
                Ok(packet) => packet,
                Err(e) => {
                    if let RecvTimeoutError::Timeout = e {
                        continue;
                    } else {
                        log::warn!("UDP Output Bridge: Failed to recv new packet: {}", e);
                        break;
                    }
                }
            };

            log::info!(
                "UDP Output Bridge: Got message from {} with size {}",
                addr,
                bytes.len()
            );

            let data_bytes: Vec<[u8; 4]> = bytes
                .into_par_iter()
                .chunks(AUDIO_VALUE_BYTE_LEN)
                .map(|chunk| {
                    let mut buf = [0u8; AUDIO_VALUE_BYTE_LEN];
                    buf.copy_from_slice(&chunk);

                    buf
                })
                .collect();

            let data: Vec<f32> = data_bytes.into_par_iter().map(f32::from_be_bytes).collect();

            match output_tx.send(data) {
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
        addresses.par_sort();
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
    pub fn udp_net(&self) -> &UdpNet {
        &self.udp_net
    }

    pub fn stop(self) {
        self.audio.stop();
        self.udp_net.stop();

        self.input_udp_bridge_handle.join();
        self.udp_output_bridge_handle.join();
    }
}
