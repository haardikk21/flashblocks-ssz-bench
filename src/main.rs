use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    time::Duration,
};

use clap::Parser;
use flate2::{Compression, write::GzEncoder};
use ssz::Encode;
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
    write: bool,
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

    if cli.file.is_none() && cli.write {
        let file_path = PathBuf::from("flashblocks.json");
        let file = File::create(&file_path).unwrap();
        serde_json::to_writer_pretty(file, &flashblocks).unwrap();
        println!("Wrote flashblocks to file: {}", &file_path.display());
    }

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
