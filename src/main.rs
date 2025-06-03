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
        .gather_flashblocks(Duration::from_secs(60))
        .await
        .unwrap();

    let json_bytes = encode_as_json(&flashblocks);
    let gzip_json_bytes = encode_as_gzip_json(&flashblocks);
    let ssz_bytes = encode_as_ssz(&flashblocks);
    let gzip_ssz_bytes = encode_as_gzip_ssz(&flashblocks);

    println!("JSON bytes: {:?}", json_bytes);
    println!("GZIP JSON bytes: {:?}", gzip_json_bytes);
    println!("SSZ bytes: {:?}", ssz_bytes);
    println!("GZIP SSZ bytes: {:?}", gzip_ssz_bytes);

    let json_to_gzip_ratio = json_bytes as f64 / gzip_json_bytes as f64;
    let json_to_ssz_ratio = json_bytes as f64 / ssz_bytes as f64;
    let json_to_gzip_ssz_ratio = json_bytes as f64 / gzip_ssz_bytes as f64;
    let ssz_to_gzip_ssz_ratio = ssz_bytes as f64 / gzip_ssz_bytes as f64;

    println!("JSON -> GZIP JSON: {:?}", json_to_gzip_ratio);
    println!("JSON -> SSZ: {:?}", json_to_ssz_ratio);
    println!("JSON -> GZIP SSZ: {:?}", json_to_gzip_ssz_ratio);
    println!("SSZ -> GZIP SSZ: {:?}", ssz_to_gzip_ssz_ratio);
}

fn encode_as_json(flashblocks: &Vec<FlashblocksPayloadV1>) -> usize {
    let serialized = serde_json::to_vec(&flashblocks).unwrap();
    serialized.len()
}

fn encode_as_gzip_json(flashblocks: &Vec<FlashblocksPayloadV1>) -> usize {
    let serialized = serde_json::to_vec(&flashblocks).unwrap();
    let mut gz_encoder = GzEncoder::new(Vec::new(), Compression::default());
    gz_encoder.write_all(&serialized).unwrap();
    let compressed = gz_encoder.finish().unwrap();

    compressed.len()
}

fn encode_as_ssz(flashblocks: &Vec<FlashblocksPayloadV1>) -> usize {
    flashblocks.as_ssz_bytes().len()
}

fn encode_as_gzip_ssz(flashblocks: &Vec<FlashblocksPayloadV1>) -> usize {
    let serialized = flashblocks.as_ssz_bytes();
    let mut gz_encoder = GzEncoder::new(Vec::new(), Compression::default());
    gz_encoder.write_all(&serialized).unwrap();
    let compressed = gz_encoder.finish().unwrap();

    compressed.len()
}
