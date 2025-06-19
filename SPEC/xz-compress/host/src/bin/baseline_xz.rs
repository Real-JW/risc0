use std::time::Instant;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CompressionResult {
    pub compressed_data: Vec<u8>,
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
    pub checksum: u32,
    pub cycles: u64, // Always 0 for baseline
}

#[derive(Debug)]
pub struct BenchmarkResult {
    pub compression_result: CompressionResult,
    pub total_time_ms: f64,
    pub throughput_mbps: f64,
}

pub struct BaselineXZ {
    compression_level: u32,
}

impl BaselineXZ {
    pub fn new(compression_level: u32) -> Self {
        Self { compression_level }
    }
    pub fn compress(&self, data: &[u8]) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let start = Instant::now();
        let compressed = {
            use std::io::Write;
            let mut encoder = xz2::write::XzEncoder::new(Vec::new(), self.compression_level);
            encoder.write_all(data)?;
            encoder.finish()?
        };
        let duration = start.elapsed();
        let total_time_ms = duration.as_secs_f64() * 1000.0;
        let throughput_mbps = (data.len() as f64 / (1024.0 * 1024.0)) / duration.as_secs_f64();
        let checksum = crc32(data);
        let result = CompressionResult {
            compressed_data: compressed.clone(),
            original_size: data.len(),
            compressed_size: compressed.len(),
            compression_ratio: data.len() as f64 / compressed.len() as f64,
            checksum,
            cycles: 0,
        };
        Ok(BenchmarkResult {
            compression_result: result,
            total_time_ms,
            throughput_mbps,
        })
    }
}

fn crc32(data: &[u8]) -> u32 {
    const CRC32_TABLE: [u32; 256] = generate_crc32_table();
    let mut crc = 0xFFFFFFFF;
    for &byte in data {
        crc = CRC32_TABLE[((crc ^ byte as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    !crc
}

const fn generate_crc32_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

fn generate_random_data(size: usize) -> Vec<u8> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut data = Vec::with_capacity(size);
    let mut hasher = DefaultHasher::new();
    for i in 0..size {
        i.hash(&mut hasher);
        data.push((hasher.finish() % 256) as u8);
    }
    data
}

fn run_benchmark(name: &str, data: &[u8], compression_level: u32) {
    println!("\n--- {} ({} bytes, level {}) ---", name, data.len(), compression_level);
    let baseline = BaselineXZ::new(compression_level);
    match baseline.compress(data) {
        Ok(result) => {
            println!("Baseline XZ:");
            println!("  Total time: {:.2} ms", result.total_time_ms);
            println!("  Compressed size: {} bytes", result.compression_result.compressed_size);
        }
        Err(e) => println!("Baseline failed: {}", e),
    }
}

fn main() {
    println!("XZ Compression Baseline Benchmark");
    println!("===============================");
    let test_cases = vec![
        ("random_data", generate_random_data(64)),
        ("random_data", generate_random_data(128)),
        ("random_data", generate_random_data(256)),
        ("random_data", generate_random_data(512)),
        ("random_data", generate_random_data(1024)),
        ("random_data", generate_random_data(2048)),
        ("random_data", generate_random_data(4096)),
        ("random_data", generate_random_data(8192)),
        ("random_data", generate_random_data(16384)),
        ("random_data", generate_random_data(32768)),
        ("random_data", generate_random_data(65536)),
        ("random_data", generate_random_data(131072)),
        ("random_data", generate_random_data(262144)),
        ("random_data", generate_random_data(524288)),
        ("random_data", generate_random_data(1048576)),
        ("random_data", generate_random_data(2097152)),
        ("random_data", generate_random_data(4194304)),
    ];
    let compression_levels = vec![9];
    for level in compression_levels {
        for (name, data) in &test_cases {
            run_benchmark(name, data, level);
        }
    }
}
