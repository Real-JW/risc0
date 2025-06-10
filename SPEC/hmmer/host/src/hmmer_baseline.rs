use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::time::Instant;

// Amino acid alphabet
const AMINO_ACIDS: &[u8] = b"ACDEFGHIKLMNPQRSTVWY";
const AA_COUNT: usize = 20;

// HMM states
#[derive(Debug, Clone, Copy)]
enum State {
    Match(()),
    Insert(()),
    Delete(()),
}

// HMM transition probabilities
#[derive(Debug, Clone)]
struct HMMTransitions {
    match_to_match: Vec<f64>,
    match_to_insert: Vec<f64>,
    match_to_delete: Vec<f64>,
    insert_to_match: Vec<f64>,
    insert_to_insert: Vec<f64>,
    delete_to_match: Vec<f64>,
    delete_to_delete: Vec<f64>,
}

// HMM emission probabilities
#[derive(Debug, Clone)]
struct HMMEmissions {
    match_emissions: Vec<[f64; AA_COUNT]>,
    insert_emissions: Vec<[f64; AA_COUNT]>,
}

// Hidden Markov Model
#[derive(Debug, Clone)]
struct HMM {
    length: usize,
    transitions: HMMTransitions,
    emissions: HMMEmissions,
}

// Protein sequence
#[derive(Debug, Clone)]
struct Sequence {
    name: String,
    data: Vec<u8>,
}

// Search result
#[derive(Debug)]
struct SearchResult {
    sequence_name: String,
    score: f64,
    e_value: f64,
    alignment_start: usize,
    alignment_end: usize,
}

impl HMM {
    fn new(length: usize) -> Self {
        let transitions = HMMTransitions {
            match_to_match: vec![0.8; length],
            match_to_insert: vec![0.1; length],
            match_to_delete: vec![0.1; length],
            insert_to_match: vec![0.5; length],
            insert_to_insert: vec![0.5; length],
            delete_to_match: vec![0.5; length],
            delete_to_delete: vec![0.5; length],
        };

        let mut match_emissions = vec![[0.05; AA_COUNT]; length];
        let insert_emissions = vec![[0.05; AA_COUNT]; length];

        // Initialize with some realistic amino acid frequencies
        let aa_freqs = [
            0.074, 0.025, 0.054, 0.062, 0.042, 0.073, 0.023, 0.052, 0.024, 0.058, 0.099, 0.045,
            0.039, 0.057, 0.073, 0.073, 0.052, 0.013, 0.034, 0.068,
        ];

        for i in 0..length {
            for j in 0..AA_COUNT {
                match_emissions[i][j] = aa_freqs[j] * (1.0 + 0.2 * (i as f64 / length as f64));
            }
        }

        let emissions = HMMEmissions {
            match_emissions,
            insert_emissions,
        };

        HMM {
            length,
            transitions,
            emissions,
        }
    }

    fn viterbi(&self, sequence: &[u8]) -> f64 {
        let seq_len = sequence.len();
        let model_len = self.length;

        // DP table: [position][state] -> score
        let mut dp = vec![vec![f64::NEG_INFINITY; model_len * 3]; seq_len + 1];
        let mut path = vec![vec![State::Match(()); model_len * 3]; seq_len + 1];

        // Initialize
        dp[0][0] = 0.0; // Start state

        // Fill DP table
        for i in 0..=seq_len {
            for j in 0..model_len {
                let match_idx = j * 3;
                let insert_idx = j * 3 + 1;
                let delete_idx = j * 3 + 2;

                if i > 0 {
                    let aa_idx = self.aa_to_index(sequence[i - 1]);

                    // Match state
                    if j > 0 {
                        let prev_match = dp[i - 1][(j - 1) * 3]
                            + self.transitions.match_to_match[j - 1].ln()
                            + self.emissions.match_emissions[j][aa_idx].ln();
                        let prev_insert = dp[i - 1][(j - 1) * 3 + 1]
                            + self.transitions.insert_to_match[j - 1].ln()
                            + self.emissions.match_emissions[j][aa_idx].ln();
                        let prev_delete = dp[i - 1][(j - 1) * 3 + 2]
                            + self.transitions.delete_to_match[j - 1].ln()
                            + self.emissions.match_emissions[j][aa_idx].ln();

                        if prev_match >= prev_insert && prev_match >= prev_delete {
                            dp[i][match_idx] = prev_match;
                            path[i][match_idx] = State::Match(());
                        } else if prev_insert >= prev_delete {
                            dp[i][match_idx] = prev_insert;
                            path[i][match_idx] = State::Insert(());
                        } else {
                            dp[i][match_idx] = prev_delete;
                            path[i][match_idx] = State::Delete(());
                        }
                    }

                    // Insert state
                    let match_to_insert = dp[i - 1][match_idx]
                        + self.transitions.match_to_insert[j].ln()
                        + self.emissions.insert_emissions[j][aa_idx].ln();
                    let insert_to_insert = dp[i - 1][insert_idx]
                        + self.transitions.insert_to_insert[j].ln()
                        + self.emissions.insert_emissions[j][aa_idx].ln();

                    if match_to_insert >= insert_to_insert {
                        dp[i][insert_idx] = match_to_insert;
                        path[i][insert_idx] = State::Match(());
                    } else {
                        dp[i][insert_idx] = insert_to_insert;
                        path[i][insert_idx] = State::Insert(());
                    }
                }

                // Delete state
                if j > 0 {
                    let match_to_delete =
                        dp[i][(j - 1) * 3] + self.transitions.match_to_delete[j - 1].ln();
                    let delete_to_delete =
                        dp[i][(j - 1) * 3 + 2] + self.transitions.delete_to_delete[j - 1].ln();

                    if match_to_delete >= delete_to_delete {
                        dp[i][delete_idx] = match_to_delete;
                        path[i][delete_idx] = State::Match(());
                    } else {
                        dp[i][delete_idx] = delete_to_delete;
                        path[i][delete_idx] = State::Delete(());
                    }
                }
            }
        }

        // Find best final score
        let mut best_score = f64::NEG_INFINITY;
        for j in 0..model_len {
            let match_score = dp[seq_len][j * 3];
            if match_score > best_score {
                best_score = match_score;
            }
        }

        best_score
    }

    fn aa_to_index(&self, aa: u8) -> usize {
        for (i, &acid) in AMINO_ACIDS.iter().enumerate() {
            if acid == aa.to_ascii_uppercase() {
                return i;
            }
        }
        0 // Default to first amino acid if not found
    }
}

fn read_sequences(filename: &str, max_sequences: usize) -> Vec<Sequence> {
    let mut sequences = Vec::new();

    match File::open(filename) {
        Ok(file) => {
            let reader = BufReader::new(file);
            let mut current_name = String::new();
            let mut current_data = Vec::new();

            for line in reader.lines() {
                if let Ok(line) = line {
                    let line = line.trim();
                    if line.starts_with('>') {
                        if !current_name.is_empty() && !current_data.is_empty() {
                            sequences.push(Sequence {
                                name: current_name.clone(),
                                data: current_data.clone(),
                            });
                            if sequences.len() >= max_sequences {
                                break;
                            }
                        }
                        current_name = line[1..].to_string();
                        current_data.clear();
                    } else {
                        current_data.extend(line.bytes());
                    }
                }
            }

            // Add last sequence
            if !current_name.is_empty()
                && !current_data.is_empty()
                && sequences.len() < max_sequences
            {
                sequences.push(Sequence {
                    name: current_name,
                    data: current_data,
                });
            }
        }
        Err(_) => {
            // Generate synthetic sequences if file doesn't exist
            println!(
                "Generating {} synthetic protein sequences...",
                max_sequences
            );
            for i in 0..max_sequences {
                let seq_len = 50 + (i % 200); // Variable length sequences
                let mut seq_data = Vec::with_capacity(seq_len);

                for j in 0..seq_len {
                    let aa_idx = (i * 17 + j * 31) % AA_COUNT;
                    seq_data.push(AMINO_ACIDS[aa_idx]);
                }

                sequences.push(Sequence {
                    name: format!("synthetic_seq_{}", i),
                    data: seq_data,
                });
            }
        }
    }

    sequences
}

fn search_sequences(hmm: &HMM, sequences: &[Sequence]) -> Vec<SearchResult> {
    let mut results = Vec::new();

    for seq in sequences {
        let score = hmm.viterbi(&seq.data);
        let e_value = (-score).exp(); // Simple E-value approximation

        results.push(SearchResult {
            sequence_name: seq.name.clone(),
            score,
            e_value,
            alignment_start: 0,
            alignment_end: seq.data.len(),
        });
    }

    // Sort by score (descending)
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    results
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse command line arguments
    let input_size = if args.len() > 1 {
        args[1].parse::<usize>().unwrap_or(1000)
    } else {
        1000
    };

    let hmm_length = if args.len() > 2 {
        args[2].parse::<usize>().unwrap_or(50)
    } else {
        50
    };

    let sequence_file = if args.len() > 3 {
        &args[3]
    } else {
        "sequences.fasta"
    };

    println!("HMMER Protein Sequence Search");
    println!("Input size: {} sequences", input_size);
    println!("HMM length: {} states", hmm_length);
    println!("Sequence file: {}", sequence_file);
    println!();

    // Initialize HMM
    let start_time = Instant::now();
    let hmm = HMM::new(hmm_length);
    println!("HMM initialized in {:?}", start_time.elapsed());

    // Read sequences
    let start_time = Instant::now();
    let sequences = read_sequences(sequence_file, input_size);
    println!(
        "Read {} sequences in {:?}",
        sequences.len(),
        start_time.elapsed()
    );

    // Perform search
    let start_time = Instant::now();
    let results = search_sequences(&hmm, &sequences);
    let search_time = start_time.elapsed();

    println!("Search completed in {:?}", search_time);
    println!("Processed {} sequences", sequences.len());
    println!(
        "Average time per sequence: {:?}",
        search_time / sequences.len() as u32
    );
    println!();

    // Display top results
    println!("Top 10 results:");
    println!(
        "{:<20} {:<12} {:<12} {:<10} {:<10}",
        "Sequence", "Score", "E-value", "Start", "End"
    );
    println!("{}", "-".repeat(70));

    for (_, result) in results.iter().take(10).enumerate() {
        println!(
            "{:<20} {:<12.2} {:<12.2e} {:<10} {:<10}",
            if result.sequence_name.len() > 20 {
                result.sequence_name[..17].to_string()
            } else {
                result.sequence_name.clone()
            },
            result.score,
            result.e_value,
            result.alignment_start,
            result.alignment_end
        );
    }

    // Write results to file
    if let Ok(mut file) = File::create("hmmer_results.txt") {
        writeln!(file, "HMMER Search Results").unwrap();
        writeln!(file, "Input size: {} sequences", input_size).unwrap();
        writeln!(file, "HMM length: {} states", hmm_length).unwrap();
        writeln!(file, "Search time: {:?}", search_time).unwrap();
        writeln!(file, "").unwrap();

        for result in &results {
            writeln!(
                file,
                "{}\t{:.2}\t{:.2e}\t{}\t{}",
                result.sequence_name,
                result.score,
                result.e_value,
                result.alignment_start,
                result.alignment_end
            )
            .unwrap();
        }
        println!("\nResults written to hmmer_results.txt");
    }

    // Performance statistics
    let total_residues: usize = sequences.iter().map(|s| s.data.len()).sum();
    let throughput = total_residues as f64 / search_time.as_secs_f64();

    println!("\nPerformance Statistics:");
    println!("Total residues processed: {}", total_residues);
    println!("Throughput: {:.0} residues/second", throughput);
    println!(
        "Memory usage estimate: ~{:.1} MB",
        (sequences.len() * 100 + hmm_length * hmm_length * 8) as f64 / 1024.0 / 1024.0
    );
}
