# Flashblocks SSZ Bench

Compares bytes length of Flashblocks payloads as encoded with JSON, Gzipped JSON, SSZ, and Gzipped SSZ.

## Usage

### Gather flashblocks from Base Sepolia

- `--duration` is specified in seconds

- `--write` is optional, and will write the gathered flashblocks to the specified file

```bash
cargo run -- --duration 60 --write flashblocks.json
```

### Reading from a file

```bash
cargo run -- --file flashblocks.json
```

## Encodings

### JSON
- Just converts `Vec<FlashblocksPayloadV1>` to a JSON byte array

### Gzipped JSON
- Converts `Vec<FlashblocksPayloadV1>` to a JSON byte array
- Uses `flate2::GzEncoder` with default compression levels to compress the byte array to a compressed version

### SSZ
- Converts `Vec<FlashblocksPayloadV1>` to an SSZ byte array, using techniques outlined in [`src/payload.rs`](./src/payload.rs)

> NOTE: The `receipts` field inside `metadata` isn't properly SSZ-encoded right now. Currently it just converts each entry in `HashMap<B256, Receipt>` to something like `[receipt_hash_b256, receipt_bytes_len, receipt_bytes]` where `receipt_bytes` is the JSON encoding of the receipt

Could probably squeeze a bit more performance here by properly encoding each receipt value to SSZ rather than just converting to JSON, needs a bit more work

### Gzipped SSZ
- Converts `Vec<FlashblocksPayloadV1>` to an SSZ byte array, using techniques outlined in [`src/payload.rs`](./src/payload.rs)
- Uses `flate2::GzEncoder` with default compression levels to compress the byte array to a compressed version