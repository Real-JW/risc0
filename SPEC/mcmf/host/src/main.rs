// host/src/main.rs
use methods::{MCMF_ELF, MCMF_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use std::time::Instant;
use mcmf::{GraphBuilder, Vertex, Capacity, Cost};
use rand::Rng;

pub fn run_mcf(n: usize) -> (i32, Vec<mcmf::Path<String>>) {
    let mut gb = GraphBuilder::new();
    let mut rng = rand::rng();

    // Generate node names: N0, N1, ..., N{n-1}
    let nodes: Vec<String> = (0..n)
        .map(|i| format!("N{}", i))
        .collect();

    // Add edges from Source to random subset of nodes
    for node in &nodes {
        if rng.random_bool(0.7) {
            gb.add_edge(
                Vertex::Source,
                node.clone(),
                Capacity(rng.random_range(1..=5)),
                Cost(rng.random_range(0..=10)),
            );
        }
    }

    // Add random edges between nodes
    for i in 0..n {
        for j in 0..n {
            if i != j && rng.random_bool(0.2) {
                gb.add_edge(
                    nodes[i].clone(),
                    nodes[j].clone(),
                    Capacity(rng.random_range(1..=5)),
                    Cost(rng.random_range(1..=100)),
                );
            }
        }
    }

    // Add edges from random nodes to Sink
    for node in &nodes {
        if rng.random_bool(0.5) {
            gb.add_edge(
                node.clone(),
                Vertex::Sink,
                Capacity(rng.random_range(1..=5)),
                Cost(rng.random_range(10..=200)),
            );
        }
    }

    let (min_cost, paths) = gb.mcmf();
    (min_cost, paths)
}


fn main() {
    // (A) Pick an “input” (for example, 10 or 25137 or whatever test‐case index you like).
    // let input: u32 = 10;
    let input: u32 = 512;
    // let input: u32 = 25137;

    // (B) Run the baseline MCF on the host, to compute (min_cost, paths).
    //     Here `run_mcf` returns (u32, Vec<Vec<u32>>).
    let compute_start = Instant::now();
    let (min_cost_i32, paths_raw) = run_mcf(input as usize);
    let compute_duration = compute_start.elapsed();
    println!("Host MCF compute time: {:?}", compute_duration);

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
    let start = Instant::now();
    let prove_info = prover.prove(env, MCMF_ELF).unwrap();
    let duration = start.elapsed();
    println!("Prover time: {:?}", duration);
    let receipt = prove_info.receipt;

    // (E) You can decode whatever the guest wrote to its journal (here: we expect it to write
    //     exactly one u32, e.g. final min_cost) if you want, or ignore that step entirely.
    let _journaled_cost: u32 = receipt.journal.decode().unwrap();

    // (F) Finally, verify the receipt on‐chain (or locally).
    let verify_start = Instant::now();  
    receipt.verify(MCMF_ID).unwrap();
    let verify_duration = verify_start.elapsed();
    println!("Verifier time: {:?}", verify_duration);
}
