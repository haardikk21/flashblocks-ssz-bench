# Flashblocks SSZ Bench

Compares bytes length of Flashblocks payloads as encoded with JSON, Gzipped JSON, SSZ, and Gzipped SSZ.

`main.rs` hardcodes a duration of 60 seconds during which it gathers real Flashblocks payloads from Base Sepolia, and then attempts encoding it in different variations and logs their byte lengths.