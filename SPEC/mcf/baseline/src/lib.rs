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

    // for (i, path) in paths.iter().enumerate() {
    //     println!(
    //         "Path {}: {:?} (cost {})",
    //         i,
    //         path.vertices(),
    //         path.cost(),
    //     );
    // }

    (min_cost, paths)
}

// fn main() {
//     let n = 25137; //The reference input for SPEC 2006’s mcf defines 25137 nodes.
//     let start = std::time::Instant::now();
//     let (min_cost, paths) = run_mcf(n);
//     let duration = start.elapsed();    
//     println!("Minimum total cost = {}", min_cost);
//     println!("Number of augmenting paths = {}", paths.len());
//     println!("Elapsed time: {:.2?}", duration);
// }
