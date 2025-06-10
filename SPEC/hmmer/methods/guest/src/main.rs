#![no_main]

use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};

// Amino acid alphabet
const AMINO_ACIDS: &[u8] = b"ACDEFGHIKLMNPQRSTVWY";
const AA_COUNT: usize = 20;

// Protein sequence structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Sequence {
    name: String,
    data: Vec<u8>,
}

// HMMER input parameters
#[derive(Debug, Serialize, Deserialize)]
struct HmmerInput {
    sequences: Vec<Sequence>,
    hmm_length: usize,
    input_size: usize,
}

// HMMER results
#[derive(Debug, Serialize, Deserialize)]
struct HmmerOutput {
    results: Vec<SearchResult>,
    total_sequences: usize,
    processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchResult {
    sequence_name: String,
    score: f64,
    e_value: f64,
    alignment_start: usize,
    alignment_end: usize,
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

        // Initialize with realistic amino acid frequencies
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

        // Simplified Viterbi for zkVM constraints
        // Using a more memory-efficient approach
        let mut prev_scores = vec![f64::NEG_INFINITY; model_len * 3];
        let mut curr_scores = vec![f64::NEG_INFINITY; model_len * 3];

        // Initialize
        prev_scores[0] = 0.0; // Start state

        // Fill DP table row by row
        for i in 0..seq_len {
            let aa_idx = self.aa_to_index(sequence[i]);

            // Reset current scores
            for j in 0..curr_scores.len() {
                curr_scores[j] = f64::NEG_INFINITY;
            }

            for j in 0..model_len {
                let match_idx = j * 3;
                let insert_idx = j * 3 + 1;
                let delete_idx = j * 3 + 2;

                // Match state transitions
                if j > 0 {
                    let prev_match_idx = (j - 1) * 3;
                    let prev_insert_idx = (j - 1) * 3 + 1;
                    let prev_delete_idx = (j - 1) * 3 + 2;

                    let match_score = self.emissions.match_emissions[j][aa_idx].ln();

                    // From previous match
                    let from_match = prev_scores[prev_match_idx]
                        + self.transitions.match_to_match[j - 1].ln()
                        + match_score;
                    // From previous insert
                    let from_insert = prev_scores[prev_insert_idx]
                        + self.transitions.insert_to_match[j - 1].ln()
                        + match_score;
                    // From previous delete
                    let from_delete = prev_scores[prev_delete_idx]
                        + self.transitions.delete_to_match[j - 1].ln()
                        + match_score;

                    curr_scores[match_idx] = from_match.max(from_insert).max(from_delete);
                }

                // Insert state transitions
                let insert_score = self.emissions.insert_emissions[j][aa_idx].ln();

                // From match to insert
                let match_to_insert = prev_scores[match_idx]
                    + self.transitions.match_to_insert[j].ln()
                    + insert_score;
                // From insert to insert
                let insert_to_insert = prev_scores[insert_idx]
                    + self.transitions.insert_to_insert[j].ln()
                    + insert_score;

                curr_scores[insert_idx] = match_to_insert.max(insert_to_insert);

                // Delete state transitions (no emission)
                if j > 0 {
                    let prev_match_idx = (j - 1) * 3;
                    let prev_delete_idx = (j - 1) * 3 + 2;

                    let match_to_delete =
                        curr_scores[prev_match_idx] + self.transitions.match_to_delete[j - 1].ln();
                    let delete_to_delete = curr_scores[prev_delete_idx]
                        + self.transitions.delete_to_delete[j - 1].ln();

                    curr_scores[delete_idx] = match_to_delete.max(delete_to_delete);
                }
            }

            // Swap score arrays
            core::mem::swap(&mut prev_scores, &mut curr_scores);
        }

        // Find best final score
        let mut best_score = f64::NEG_INFINITY;
        for j in 0..model_len {
            let match_score = prev_scores[j * 3];
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
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(core::cmp::Ordering::Equal)
    });
    results
}

// Simple timing function for guest environment
fn get_time_ms() -> u64 {
    // In a real implementation, you might want to use cycle counting
    // For now, we'll use a placeholder
    0
}

risc0_zkvm::entry!(main);

fn main() {
    // Read input data from host
    let input: HmmerInput = env::read();

    let start_time = get_time_ms();

    // Initialize HMM
    let hmm = HMM::new(input.hmm_length);

    // Perform search
    let results = search_sequences(&hmm, &input.sequences);

    let end_time = get_time_ms();
    let processing_time = end_time.saturating_sub(start_time);

    // Create output
    let output = HmmerOutput {
        results,
        total_sequences: input.sequences.len(),
        processing_time_ms: processing_time,
    };

    // Commit the results to the journal
    env::commit(&output);
}
