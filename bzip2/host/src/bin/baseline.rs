use bzip2::read::{BzDecoder, BzEncoder};
use bzip2::Compression;
use std::io::Read;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Prepare the raw data (for example purposes, repeat a short message 100×)
    let raw: Vec<u8> = b"Hi there, this is Jiawen!\n"
        .repeat(4096)
        .into();

    // Compression timer
    let compress_start = Instant::now();
    let mut encoder = BzEncoder::new(&raw[..], Compression::best());
    let mut compressed: Vec<u8> = Vec::new();
    encoder
        .read_to_end(&mut compressed)
        .expect("Host‐side bzip2 compression failed");
    let compress_duration = compress_start.elapsed();

    println!(
        "Compressed {} → {} bytes in {:?}",
        raw.len(),
        compressed.len(),
        compress_duration
    );

    // Decompression timer
    let decompress_start = Instant::now();
    let mut decoder = BzDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    let decompress_duration = decompress_start.elapsed();

    assert_eq!(decompressed, raw);
    println!(
        "Decompression OK ({} bytes) in {:?}",
        decompressed.len(),
        decompress_duration
    );

    Ok(())
}
