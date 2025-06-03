use risc0_zkvm::guest::env;
use bzip2::read::BzDecoder;
use std::io::Read;

fn main() {
    // 1. Read the single byte‐buffer that host passed in:
    //    [ 8 bytes little‐endian raw_len ∥ raw_len bytes of `raw` ∥ rest = bzip2‐compressed ]
    let input_data: Vec<u8> = env::read();

    // 2. Parse out the first 8 bytes as a little‐endian u64 → raw_len
    assert!(
        input_data.len() >= 8,
        "Expected at least 8 bytes to read raw_len"
    );
    let raw_len = {
        let mut len_bytes = [0u8; 8];
        len_bytes.copy_from_slice(&input_data[0..8]);
        u64::from_le_bytes(len_bytes) as usize
    };

    // 3. Sanity check: we must have at least (8 + raw_len) bytes
    assert!(
        input_data.len() >= 8 + raw_len,
        "Buffer too short: raw_len = {}, but total bytes = {}",
        raw_len,
        input_data.len()
    );

    // 4. Split into `raw` and `compressed`
    let raw = &input_data[8..8 + raw_len];
    let compressed = &input_data[8 + raw_len..];

    // 5. Decompress `compressed` and compare to `raw`
    let mut decoder = BzDecoder::new(&compressed[..]);
    let mut decompressed: Vec<u8> = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .expect("Bzip2 decompression failed in guest");

    // 6. If decompression did not match the original, panic (proof will fail)
    assert_eq!(
        decompressed, raw,
        "Decompressed payload did not match the original raw bytes!"
    );

    // 7. Commit a single success‐byte to the journal (optional, but shows "I succeeded")
    env::commit(&[1u8]);
}
