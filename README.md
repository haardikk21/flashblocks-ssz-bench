# Flashblocks SSZ Bench

Compares bytes length of Flashblocks payloads as encoded with JSON, Gzipped JSON, SSZ, and Gzipped SSZ.

## Usage

### Gather flashblocks from Base Sepolia

Duration is specified in seconds

`--write` will write the gathered flashblocks in a local `flashblocks.json` file

```bash
cargo run --
    --duration 60
    --write
```

### Reading from a file

```bash
cargo run --
    --file flashblocks.json
```