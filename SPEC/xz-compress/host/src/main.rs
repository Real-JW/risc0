use std::time::{Instant};
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde::{Deserialize, Serialize};
use methods::{XZ_COMPRESS_ELF, XZ_COMPRESS_ID};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CompressionInput {
    pub data: Vec<u8>,
    pub compression_level: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CompressionResult {
    pub compressed_data: Vec<u8>,
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
    pub checksum: u32,
    pub cycles: u64,
}

#[derive(Debug)]
pub struct BenchmarkResult {
    pub compression_result: CompressionResult,
    pub prove_time_ms: f64,
    pub verify_time_ms: f64,
    pub total_time_ms: f64,
    pub throughput_mbps: f64,
}

// Baseline implementation using standard xz2
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
            cycles: 0, // N/A for baseline
        };
        
        Ok(BenchmarkResult {
            compression_result: result,
            prove_time_ms: 0.0, // N/A for baseline
            verify_time_ms: 0.0, // N/A for baseline
            total_time_ms,
            throughput_mbps,
        })
    }
}

// RISC Zero implementation
pub struct RiscZeroXZ;

impl RiscZeroXZ {
    pub fn new() -> Self {
        Self
    }
    
    pub fn compress(&self, data: &[u8], compression_level: u32) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let input = CompressionInput {
            data: data.to_vec(),
            compression_level,
        };
        
        // Prove phase
        let prove_start = Instant::now();
        
        let env = ExecutorEnv::builder()
            .write(&input)?
            .build()?;
        
        let prover = default_prover();
        let prove_info = prover.prove(env, XZ_COMPRESS_ELF).unwrap();
        let prove_time = prove_start.elapsed();
        let prove_time_ms = prove_time.as_secs_f64() * 1000.0;
        let receipt = prove_info.receipt;
        
        // Verify phase
        let verify_start = Instant::now();
        receipt.verify(XZ_COMPRESS_ID)?;
        let verify_time = verify_start.elapsed();
        let verify_time_ms = verify_time.as_secs_f64() * 1000.0;
        
        let total_time_ms = prove_time_ms + verify_time_ms;
        let throughput_mbps = (data.len() as f64 / (1024.0 * 1024.0)) / (total_time_ms / 1000.0);
        
        let compression_result: CompressionResult = receipt.journal.decode().unwrap();
        
        Ok(BenchmarkResult {
            compression_result,
            prove_time_ms,
            verify_time_ms,
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

// Test data generators
// fn generate_text_data(size: usize) -> Vec<u8> {
//     let text = "The quick brown fox jumps over the lazy dog. This is sample text for compression testing. ";
//     let mut data = Vec::new();
//     while data.len() < size {
//         data.extend_from_slice(text.as_bytes());
//     }
//     data.truncate(size);
//     data
// }

// fn generate_binary_data(size: usize) -> Vec<u8> {
//     (0..size).map(|i| (i % 256) as u8).collect()
// }

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
    
    // Baseline benchmark
    let baseline = BaselineXZ::new(compression_level);
    match baseline.compress(data) {
        Ok(result) => {
            println!("Baseline XZ:");
            println!("  Total time: {:.2} ms", result.total_time_ms);
            // println!("  Throughput: {:.2} MB/s", result.throughput_mbps);
            // println!("  Compression ratio: {:.2}x", result.compression_result.compression_ratio);
            println!("  Compressed size: {} bytes", result.compression_result.compressed_size);
        }
        Err(e) => println!("Baseline failed: {}", e),
    }
    
    // RISC Zero benchmark
    let risc_zero = RiscZeroXZ::new();
    match risc_zero.compress(data, compression_level) {
        Ok(result) => {
            println!("RISC Zero XZ:");
            println!("  Prove time: {:.2} ms", result.prove_time_ms);
            println!("  Verify time: {:.2} ms", result.verify_time_ms);
            println!("  Total time: {:.2} ms", result.total_time_ms);
            // println!("  Throughput: {:.2} MB/s", result.throughput_mbps);
            println!("  Compression ratio: {:.2}x", result.compression_result.compression_ratio);
            println!("  Compressed size: {} bytes", result.compression_result.compressed_size);
            println!("  Guest cycles: {}", result.compression_result.cycles);
        }
        Err(e) => println!("RISC Zero failed: {}", e),
    }
}

fn main() {
    println!("XZ Compression Benchmark - RISC Zero vs Baseline");
    println!("================================================");
    
    let test_cases = vec![
        ("random_data", generate_random_data(64)),        // 64B
        ("random_data", generate_random_data(128)),        // 128B
        ("random_data", generate_random_data(256)),        // 256B
        ("random_data", generate_random_data(512)),        // 512B
        ("random_data", generate_random_data(1024)),        // 1KB
        ("random_data", generate_random_data(2048)),        // 2KB
        ("random_data", generate_random_data(4096)),        // 4KB
        ("random_data", generate_random_data(8192)),        // 8KB
        ("random_data", generate_random_data(16384)),        // 16KB
        ("random_data", generate_random_data(32768)),        // 32KB
        ("random_data", generate_random_data(65536)),        // 64KB
        ("random_data", generate_random_data(131072)),        // 128KB
        ("random_data", generate_random_data(262144)),        // 256KB
        ("random_data", generate_random_data(524288)),        // 512KB
        ("random_data", generate_random_data(1048576)),        // 1MB
        ("random_data", generate_random_data(2097152)),        // 2MB
        ("random_data", generate_random_data(4194304)),        // 4MB
    ];
    
    let compression_levels = vec![9];
    
    for level in compression_levels {
        for (name, data) in &test_cases {
            run_benchmark(name, data, level);
        }
    }
}