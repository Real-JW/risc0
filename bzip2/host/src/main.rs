use methods::{METHOD_ELF, METHOD_ID};
use risc0_zkvm::{ExecutorEnv, default_prover};
use bzip2::read::BzEncoder;
use bzip2::Compression;
use std::io::Read;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Prepare the raw data (for example purposes, repeat a short message 100×)
    let raw: Vec<u8> = b"Hi there, this is Jiawen!\n"
        .repeat(16384)
        .into();

    // 2. Compress `raw` with bzip2 on the host side
    let mut encoder = BzEncoder::new(&raw[..], Compression::best());
    let mut compressed: Vec<u8> = Vec::new();
    encoder
        .read_to_end(&mut compressed)
        .expect("Host‐side bzip2 compression failed");

    println!("Compressed {} → {} bytes", raw.len(), compressed.len());

    // 3. Build a single “to_guest” buffer = [ raw_len (8 bytes LE) ∥ raw ∥ compressed ]
    let raw_len = raw.len() as u64;
    let mut to_guest: Vec<u8> = Vec::new();
    to_guest.extend_from_slice(&raw_len.to_le_bytes()); // 8 bytes
    to_guest.extend_from_slice(&raw);                   // raw_len bytes
    to_guest.extend_from_slice(&compressed);            // remaining bytes

    // 4. Build the ExecutorEnv by writing that full buffer
    let mut builder = ExecutorEnv::builder();
    builder.write(&to_guest)?;
    let env = builder.build()?;

    // 5. Run the prover (guest will decompress+verify inside ZK‐VM)
    println!("Prover time…");
    let start = Instant::now();
    let prover = default_prover();
    let info = prover.prove(env, METHOD_ELF)?;
    let duration = start.elapsed();
    println!("Prover completed in {:?}", duration);

    // 6. Verify the proof on the host
    println!("Verifier time…");
    let verify_start = Instant::now();
    info.receipt.verify(METHOD_ID)?;
    let verify_duration = verify_start.elapsed();
    println!("Verifier completed in {:?}", verify_duration);

    // 7. Inspect the journal. In our guest, we committed a single byte [1].
    let journal_bytes: Vec<u8> = info.receipt.journal.bytes.clone();
    println!(
        "Journal from guest (should be [1]): {:?}",
        journal_bytes
    );

    Ok(())
}
