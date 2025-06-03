use std::{io::Write, time::Duration};

use flate2::{Compression, write::GzEncoder};
use ssz::Encode;
use tokio_tungstenite::tungstenite::http::Uri;

use crate::{payload::FlashblocksPayloadV1, subscriber::WebsocketSubscriber};

mod payload;
mod subscriber;

#[tokio::main]
async fn main() {
    let subscriber =
        WebsocketSubscriber::new(Uri::from_static("wss://sepolia.flashblocks.base.org/ws"));
    let flashblocks = subscriber
        .gather_flashblocks(Duration::from_secs(10))
        .await
        .unwrap();

    let _ = encode_as_json(&flashblocks);
    let _ = encode_as_gzip_json(&flashblocks);
    let _ = encode_as_ssz(&flashblocks);
    let _ = encode_as_gzip_ssz(&flashblocks);
}

fn encode_as_json(flashblocks: &Vec<FlashblocksPayloadV1>) -> Vec<u8> {
    let serialized = serde_json::to_vec(&flashblocks).unwrap();
    println!("JSON bytes len: {:?}", serialized.len());
    serialized
}

fn encode_as_gzip_json(flashblocks: &Vec<FlashblocksPayloadV1>) -> Vec<u8> {
    let serialized = serde_json::to_vec(&flashblocks).unwrap();
    let mut gz_encoder = GzEncoder::new(Vec::new(), Compression::default());
    gz_encoder.write_all(&serialized).unwrap();
    let compressed = gz_encoder.finish().unwrap();

    println!("GZIP JSON bytes len: {:?}", compressed.len());

    compressed
}

fn encode_as_ssz(flashblocks: &Vec<FlashblocksPayloadV1>) -> Vec<u8> {
    let serialized = flashblocks.as_ssz_bytes();
    println!("SSZ bytes len: {:?}", serialized.len());
    serialized
}

fn encode_as_gzip_ssz(flashblocks: &Vec<FlashblocksPayloadV1>) -> Vec<u8> {
    let serialized = flashblocks.as_ssz_bytes();
    let mut gz_encoder = GzEncoder::new(Vec::new(), Compression::default());
    gz_encoder.write_all(&serialized).unwrap();
    let compressed = gz_encoder.finish().unwrap();

    println!("GZIP SSZ bytes len: {:?}", compressed.len());
    compressed
}
