// host/src/main.rs
use baseline::run_mcf;
use methods::{MCMF_ELF, MCMF_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};

fn main() {
    // (A) Pick an “input” (for example, 10 or 25137 or whatever test‐case index you like).
    let input: u32 = 10;
    // let input: u32 = 25137;

    // (B) Run the baseline MCF on the host, to compute (min_cost, paths).
    //     Here `run_mcf` returns (u32, Vec<Vec<u32>>).
    let (min_cost_i32, paths_raw) = run_mcf(input as usize);

    // Convert min_cost to u32 and paths to Vec<Vec<u32>>
    let min_cost = min_cost_i32 as u32;
    let paths: Vec<Vec<u32>> = paths_raw
        .into_iter()
        .map(|path| {
            path.vertices()
                .iter()
                .filter_map(|v| {
                    // Match Vertex::Node(String) and parse "N123" to 123
                    if let mcmf::Vertex::Node(ref name) = v {
                        if let Some(stripped) = name.strip_prefix("N") {
                            stripped.parse::<u32>().ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect();

    // (C) Build an ExecutorEnv and serialize (input, min_cost, paths) into it, in that exact order.
    let env = ExecutorEnv::builder()
        .write(&input)     // 1st: the u32 “input”
        .unwrap()
        .write(&min_cost)  // 2nd: the u32 “min_cost”
        .unwrap()
        .write(&paths)     // 3rd: the Vec<Vec<u32>> “paths”
        .unwrap()
        .build()
        .unwrap();

    // (D) Prove against your guest ELF:
    let prover = default_prover();
    let prove_info = prover.prove(env, MCMF_ELF).unwrap();
    let receipt = prove_info.receipt;

    // (E) You can decode whatever the guest wrote to its journal (here: we expect it to write
    //     exactly one u32, e.g. final min_cost) if you want, or ignore that step entirely.
    let _journaled_cost: u32 = receipt.journal.decode().unwrap();

    // (F) Finally, verify the receipt on‐chain (or locally).
    receipt.verify(MCMF_ID).unwrap();
}