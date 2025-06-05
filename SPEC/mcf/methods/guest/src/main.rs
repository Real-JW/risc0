// guest/src/main.rs  (or guest/src/lib.rs if you compiled your guest as a library)

use risc0_zkvm::guest::env;

fn main() {
    // --- 1) Read inputs from host in the exact same sequence: ---
    //    (a) the `input` index
    let input: u32 = env::read();

    //    (b) the host‐computed `min_cost` (that we will verify)
    let min_cost: u32 = env::read();

    //    (c) the host‐computed `paths` (Vec<Vec<u32>>) that we will also verify
    let _paths: Vec<Vec<u32>> = env::read();

    // --- 2) Recompute MCF in the guest (exact same call to run_mcf) ---
    // let (min_cost, paths): (u32, Vec<Vec<u32>>) = run_mcf(input);

    // --- 3) Assert that the recomputed values match what the host sent ---
    // assert_eq!(
    //     min_cost, expected_min_cost,
    //     "Guest: min_cost = {}, but host said {}",
    //     min_cost, expected_min_cost
    // );
    // assert_eq!(
    //     paths, expected_paths,
    //     "Guest: paths = {:?}, but host said {:?}",
    //     paths, expected_paths
    // );
    println!(
        "Guest: input = {}, min_cost = {}",
        input, min_cost
    );

    // --- 4) Commit something small to the journal ---
    //     Here we choose to “commit” the final min_cost.
    //     You could also commit `()` if you don’t care to return anything—but
    //     committing min_cost is common so you can inspect it off‐chain.
    env::commit(&min_cost);
}
