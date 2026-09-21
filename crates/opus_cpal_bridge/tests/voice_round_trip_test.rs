use opus_cpal_bridge::stream::{
    Stream,
    stream_inner::{Input, Output},
};
use ringbuf::traits::{Consumer, Producer};

const RINGBUF_SIZE: usize = 4 * 1024;

#[test]
fn voice_round_trip() {
    tracing_subscriber::fmt().init();

    let host = cpal::default_host();
    let (_input, mut consumer) = Stream::<Input>::new(&host, RINGBUF_SIZE).unwrap();
    let (_output, mut producer) = Stream::<Output>::new(&host, RINGBUF_SIZE).unwrap();

    loop {
        let count = producer.push_iter(consumer.pop_iter());
        tracing::info!("Moved {} samples from Input to Output.", count);
        std::thread::park_timeout(std::time::Duration::from_millis(5));
    }
}
