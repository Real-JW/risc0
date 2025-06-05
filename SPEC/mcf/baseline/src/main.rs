use baseline::run_mcf;
use std::time::Instant;

fn main() {
    // Example: pick some input value to test
    let input: usize = 25137;

    // Call the library function
    let start = Instant::now();
    let (min_cost, _paths) = run_mcf(input);
    let duration = start.elapsed();
    println!("run_mcf took {:?}", duration);

    // Print the result to stdout
    println!("For input = {input}, min_cost = {min_cost}");
}