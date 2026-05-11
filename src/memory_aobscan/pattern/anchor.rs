//! Intelligent Anchor Selection
//!
//! Provides heuristic anchor selection to minimize false positives during scan.
//! Selects rarest single byte or best multi-byte sequence based on rarity analysis.
//!
//! # Optimization Principles
//! - **Rarest byte selection**: Reduces memchr false positives
//! - **Multi-byte sequences**: Further reduces verification calls
//! - **Consecutive bytes only**: For predictable offset relationships

/// Finds the best multi-byte anchor sequence (2-4 consecutive known bytes).
///
/// Optimizes search performance by selecting sequence with lowest total
/// byte frequency to minimize false positives.
///
/// # Arguments
/// * `bytes` - Pattern byte array
/// * `mask` - Pattern mask array (true = known byte, false = wildcard)
///
/// # Returns
/// * `Some(Vec<(usize, u8)>)` - Best anchor sequence as (offset, byte) pairs
/// * `None` - If no suitable sequence found (less than 2 consecutive known bytes)
pub fn find_best_anchor_sequence(bytes: &[u8], mask: &[bool]) -> Option<Vec<(usize, u8)>> {
    let mut best_sequence: Option<Vec<(usize, u8)>> = None;
    let mut best_score = u32::MAX;

    // precompute frequency of all known bytes
    let mut freq = [0u32; 256];
    for (i, &byte) in bytes.iter().enumerate() {
        if mask[i] {
            freq[byte as usize] += 1;
        }
    }

    // Find all consecutive known byte sequences (length 2-4)
    let mut current_seq = Vec::new();

    for (i, &byte) in bytes.iter().enumerate() {
        if mask[i] {
            current_seq.push((i, byte));

            if current_seq.len() >= 2 && current_seq.len() <= 4 {
                // Calculate rarity score for this sequence
                let freq_score: u32 = current_seq.iter().map(|(_, b)| freq[*b as usize]).sum();
                // calculate continuity bonus (more consecutive bytes are better)
                let mut continuity_bonus = 0;
                for j in 1..current_seq.len() {
                    if current_seq[j].0 == current_seq[j - 1].0 + 1 {
                        continuity_bonus += 1;
                    }
                }

                let score = freq_score - continuity_bonus as u32;

                if score < best_score {
                    best_score = score;
                    best_sequence = Some(current_seq.clone());
                }
            }

            if current_seq.len() == 4 {
                // Remove oldest to maintain max length of 4
                current_seq.remove(0);
            }
        } else {
            // Reset on wildcard
            current_seq.clear();
        }
    }

    best_sequence
} 

/// Finds the index of the most rare non-wildcard byte for heuristic searching.
///
/// Optimizes search performance by minimizing false positives from memchr.
/// Uses frequency analysis to select least common byte in pattern.
///
/// # Arguments
/// * `bytes` - Pattern byte array
/// * `mask` - Pattern mask array
///
/// # Returns
/// * `Some(usize)` - Index of the rarest non-wildcard byte
/// * `None` - If pattern contains only wildcards
///
/// # Example
/// ```ignore
/// let bytes = vec![0x48, 0x89, 0x48, 0x55];
/// let mask = vec![true, true, true, true];
/// // 0x48 appears twice, 0x89 and 0x55 appear once
/// // Returns index of either 0x89 or 0x55 (the rarer ones)
/// ```
pub fn find_rarest_byte_index(bytes: &[u8], mask: &[bool]) -> Option<usize> {
    if bytes.is_empty() {
        return None;
    }

    // Count frequency of each byte value in the pattern
    let mut freq = [0u32; 256];
    for (i, &byte) in bytes.iter().enumerate() {
        if mask[i] {
            freq[byte as usize] += 1;
        }
    }

    // Find the non-wildcard byte with lowest frequency
    mask.iter()
        .enumerate()
        .filter(|(_, &m)| m)
        .min_by_key(|(i, _)| freq[bytes[*i] as usize])
        .map(|(i, _)| i)
}
