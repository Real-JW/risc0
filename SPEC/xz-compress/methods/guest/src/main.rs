use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};

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

// Simplified LZMA-style compression for zkVM
fn simple_lzma_compress(data: &[u8], level: u32) -> Vec<u8> {
    let mut compressed = Vec::new();
    let window_size = match level {
        1..=3 => 1024,   // Small window for fast compression
        4..=6 => 2048,   // Medium window
        7..=9 => 4096,   // Large window for better compression
        _ => 2048,
    };
    
    let mut i = 0;
    while i < data.len() {
        let mut best_match = (0, 0); // (distance, length)
        let search_start = if i >= window_size { i - window_size } else { 0 };
        
        // Look for matches in the sliding window
        for j in search_start..i {
            let mut match_len = 0;
            while i + match_len < data.len() 
                && j + match_len < i 
                && data[j + match_len] == data[i + match_len] 
                && match_len < 258 {
                match_len += 1;
            }
            
            if match_len >= 3 && match_len > best_match.1 {
                best_match = (i - j, match_len);
            }
        }
        
        if best_match.1 >= 3 {
            // Encode match: flag + distance (2 bytes) + length (1 byte)
            compressed.push(0xFF); // Match flag
            compressed.extend_from_slice(&(best_match.0 as u16).to_le_bytes());
            compressed.push(best_match.1 as u8);
            i += best_match.1;
        } else {
            // Literal byte
            compressed.push(0x00); // Literal flag
            compressed.push(data[i]);
            i += 1;
        }
    }
    
    compressed
}

fn crc32(data: &[u8]) -> u32 {
    // Simplified CRC32 implementation
    const CRC32_POLY: u32 = 0xEDB88320;
    let mut crc = 0xFFFFFFFF;
    
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ CRC32_POLY;
            } else {
                crc >>= 1;
            }
        }
    }
    
    !crc
}

fn main() {
    let start_cycles = env::cycle_count();
    
    // Read input from host
    let input: CompressionInput = env::read();
    
    // Perform compression
    let compressed_data = simple_lzma_compress(&input.data, input.compression_level);
    let checksum = crc32(&input.data);
    
    let end_cycles = env::cycle_count();
    let total_cycles = end_cycles - start_cycles;
    
    let result = CompressionResult {
        compressed_data: compressed_data.clone(),
        original_size: input.data.len(),
        compressed_size: compressed_data.len(),
        compression_ratio: input.data.len() as f64 / compressed_data.len() as f64,
        checksum,
        cycles: total_cycles,
    };
    
    // Commit result to journal
    env::commit(&result);
}