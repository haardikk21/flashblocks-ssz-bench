use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::Parser;
use flate2::{Compression, write::GzEncoder};
use futures_util::future::join_all;
use ssz::Encode;
use tokio::task;
use tokio_tungstenite::tungstenite::http::Uri;

use crate::{payload::FlashblocksPayloadV1, subscriber::WebsocketSubscriber};

mod payload;
mod subscriber;

#[derive(Parser)]
#[command(name = "flashblocks-ssz-bench")]
#[command(
    about = "Compares bytes length of Flashblocks payloads as encoded with JSON, Gzipped JSON, SSZ, and Gzipped SSZ"
)]
struct Cli {
    /// Duration in seconds to gather flashblocks (only used with --gather)
    #[arg(short = 'd', long = "duration", default_value = "60")]
    duration: u64,

    /// Read flashblocks from a local JSON file
    #[arg(short = 'f', long = "file")]
    file: Option<PathBuf>,

    /// Write gathered flashblocks to a local JSON file
    #[arg(short = 'w', long = "write")]
    write: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let flashblocks = if let Some(file_path) = &cli.file {
        // Read from file
        println!("Reading flashblocks from file: {}", file_path.display());
        let file_content = fs::read_to_string(&file_path)
            .unwrap_or_else(|e| panic!("Failed to read file {}: {}", file_path.display(), e));

        serde_json::from_str::<Vec<FlashblocksPayloadV1>>(&file_content)
            .unwrap_or_else(|e| panic!("Failed to parse JSON from file: {}", e))
    } else {
        // Default to gather mode if no file specified
        println!("No file specified, defaulting to gather mode");
        let subscriber =
            WebsocketSubscriber::new(Uri::from_static("wss://sepolia.flashblocks.base.org/ws"));
        subscriber
            .gather_flashblocks(Duration::from_secs(cli.duration))
            .await
            .unwrap()
    };

    println!("Loaded {} flashblocks", flashblocks.len());

    if cli.file.is_none() && cli.write.is_some() {
        let file_path = PathBuf::from(cli.write.unwrap());
        let file = File::create(&file_path).unwrap();
        serde_json::to_writer_pretty(file, &flashblocks).unwrap();
        println!("Wrote flashblocks to file: {}", &file_path.display());
    }
    println!("");
    let tasks = vec![
        ("JSON", task::spawn(encode_as_json(flashblocks.clone()))),
        (
            "gzip JSON",
            task::spawn(encode_as_gzip_json(flashblocks.clone())),
        ),
        (
            "brotli JSON",
            task::spawn(encode_as_brotli_json(flashblocks.clone())),
        ),
        ("SSZ", task::spawn(encode_as_ssz(flashblocks.clone()))),
        (
            "gzip SSZ",
            task::spawn(encode_as_gzip_ssz(flashblocks.clone())),
        ),
        (
            "brotli SSZ",
            task::spawn(encode_as_brotli_ssz(flashblocks.clone())),
        ),
    ];

    let results = join_all(tasks.into_iter().map(|(label, handle)| async move {
        let result = handle.await;
        (label, result.expect("Failed to get result"))
    }))
    .await;

    let mut json_bytes: usize = 0;
    let mut ssz_bytes: usize = 0;
    for (label, (bytes, duration)) in results.clone() {
        if label == "JSON" {
            json_bytes = bytes;
        } else if label == "SSZ" {
            ssz_bytes = bytes;
        }

        println!("{}: {:?} bytes in {:?}", label, bytes, duration);
    }

    println!("");
    for (label, (bytes, _)) in results.clone() {
        if label != "JSON" {
            let ratio = json_bytes as f64 / bytes as f64;
            println!("JSON -> {}: {:.3}x improvement", label, ratio);
        }

        if label.contains("SSZ") && label != "SSZ" {
            let ratio = ssz_bytes as f64 / bytes as f64;
            println!("SSZ -> {}: {:.3}x improvement", label, ratio);
        }
    }
}

async fn encode_as_json(flashblocks: Vec<FlashblocksPayloadV1>) -> (usize, Duration) {
    let start_time = Instant::now();
    let serialized = serde_json::to_vec(&flashblocks).unwrap();
    (serialized.len(), start_time.elapsed())
}

async fn encode_as_gzip_json(flashblocks: Vec<FlashblocksPayloadV1>) -> (usize, Duration) {
    let start_time = Instant::now();
    let serialized = serde_json::to_vec(&flashblocks).unwrap();
    let mut gz_encoder = GzEncoder::new(Vec::new(), Compression::default());
    gz_encoder.write_all(&serialized).unwrap();
    let compressed = gz_encoder.finish().unwrap();

    (compressed.len(), start_time.elapsed())
}

async fn encode_as_brotli_json(flashblocks: Vec<FlashblocksPayloadV1>) -> (usize, Duration) {
    let start_time = Instant::now();
    let serialized = serde_json::to_vec(&flashblocks).unwrap();
    let mut compressed = Vec::new();
    {
        let mut compressor = brotli::CompressorWriter::new(&mut compressed, 4096, 5, 22);
        compressor.write_all(&serialized).unwrap();
    }
    (compressed.len(), start_time.elapsed())
}

async fn encode_as_ssz(flashblocks: Vec<FlashblocksPayloadV1>) -> (usize, Duration) {
    let start_time = Instant::now();
    (flashblocks.as_ssz_bytes().len(), start_time.elapsed())
}

async fn encode_as_gzip_ssz(flashblocks: Vec<FlashblocksPayloadV1>) -> (usize, Duration) {
    let start_time = Instant::now();
    let serialized = flashblocks.as_ssz_bytes();
    let mut gz_encoder = GzEncoder::new(Vec::new(), Compression::default());
    gz_encoder.write_all(&serialized).unwrap();
    let compressed = gz_encoder.finish().unwrap();

    (compressed.len(), start_time.elapsed())
}

async fn encode_as_brotli_ssz(flashblocks: Vec<FlashblocksPayloadV1>) -> (usize, Duration) {
    let start_time = Instant::now();
    let serialized = flashblocks.as_ssz_bytes();
    let mut compressed = Vec::new();
    {
        let mut compressor = brotli::CompressorWriter::new(&mut compressed, 4096, 5, 22);
        compressor.write_all(&serialized).unwrap();
    }
    (compressed.len(), start_time.elapsed())
}
