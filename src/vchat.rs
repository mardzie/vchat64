use std::{
    net::{SocketAddr, ToSocketAddrs},
    sync::{Arc, Mutex, RwLock},
    thread::{self, JoinHandle},
};

use color_eyre::eyre::Result;
use crossbeam::channel::{Receiver, Sender};
use ringbuf::traits::{Consumer, Observer};

use crate::{
    audio::{Audio, InputMessage, audio_processor::AudioProcessor, traits::SampleFormatConversion},
    types::{ArcMutex, ArcRwLock},
    udp_packet_net::MAX_PAYLOAD_SIZE,
    voice_net::{self, VoiceNet},
};

pub const AUDIO_CHANNELS_BUF_SIZE: usize = 1024 * 16;

pub struct VChat {
    audio: Arc<Audio>, // TODO: Input Type
    voice_net: ArcMutex<VoiceNet>,
    addresses: ArcRwLock<Vec<SocketAddr>>,

    exit_notify: crossbeam::channel::Sender<InputMessage>,
    udp_bridge_handle: JoinHandle<()>,
}

impl VChat {
    pub fn new<A>(addr: A) -> Result<Self>
    where
        A: ToSocketAddrs,
    {
        let (exit_notify, input_notify_rx) = crossbeam::channel::bounded(128);
        let (speaker_input_tx, speaker_output_rx) =
            crossbeam::channel::bounded(AUDIO_CHANNELS_BUF_SIZE);

        let input_notify_tx_c = exit_notify.clone();
        let (audio, consumer) = Audio::new(input_notify_tx_c, speaker_output_rx, u8::MAX);
        let audio = Arc::new(audio);

        let voice_net = Arc::new(Mutex::new(
            VoiceNet::new(addr).expect("Failed to create VoiceNet."),
        ));

        let addresses = Arc::new(RwLock::new(Vec::with_capacity(8)));

        let audio_c = audio.clone();
        let voice_net_c = voice_net.clone();
        let addresses_c = addresses.clone();
        let udp_bridge_handle = thread::spawn(move || {
            Self::udp_bridge(
                voice_net_c,
                addresses_c,
                input_notify_rx,
                consumer,
                speaker_input_tx,
                audio_c,
                output_sample_format,
            )
        });

        audio.play();

        Ok(Self {
            audio,
            voice_net,
            addresses,

            exit_notify,
            udp_bridge_handle,
        })
    }

    fn udp_bridge<T>(
        voice_net: ArcMutex<VoiceNet>,
        addresses: ArcRwLock<Vec<SocketAddr>>,

        input_notify: Receiver<InputMessage>,
        mut input_ringbuf: ringbuf::wrap::caching::Caching<
            Arc<ringbuf::SharedRb<ringbuf::storage::Heap<f32>>>,
            false,
            true,
        >,
        output_tx: Sender<Vec<f32>>,
        audio: Arc<Audio>,
    ) where
        T: Copy,
    {
        let output_sample_format = audio.output_sample_format();
        let audio_processor = audio.audio_processor();

        loop {
            if Self::input_udp_bridge(
                &voice_net,
                &audio_processor,
                &input_notify,
                &mut input_ringbuf,
                &addresses,
            )
            .is_err()
            {
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
        audio_processor: &Arc<AudioProcessor>,
        input_notify: &Receiver<InputMessage>,
        input_ringbuf: &mut ringbuf::wrap::caching::Caching<
            Arc<ringbuf::SharedRb<ringbuf::storage::Heap<f32>>>,
            false,
            true,
        >,
        addresses: &ArcRwLock<Vec<SocketAddr>>,
    ) -> Result<(), ()> {
        match input_notify.try_recv() {
            Ok(InputMessage::Samples) => {}
            Err(crossbeam::channel::TryRecvError::Empty) => return Ok(()),
            Ok(InputMessage::Exit) => return Err(()),
            Err(crossbeam::channel::TryRecvError::Disconnected) => return Err(()),
        };

        let mut data = Vec::with_capacity(input_ringbuf.occupied_len());
        input_ringbuf.pop_slice(&mut data);

        let data = audio_processor.process_audio(data);

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

    pub fn remove_address(&self, addr: &SocketAddr) -> Option<SocketAddr> {
        let mut lock = self
            .addresses
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let idx = lock
            .iter()
            .position(|internal_addr| addr == internal_addr)?;
        Some(lock.remove(idx))
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
        if let Err(e) = self.exit_notify.send(InputMessage::Exit) {
            log::error!("Failed to send exit notification to audio thread: {}", e);
        };

        let _ = self.udp_bridge_handle.join();
    }
}
