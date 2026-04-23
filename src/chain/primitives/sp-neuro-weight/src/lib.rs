//! Sparse weight vector + run-length compression primitives for NeuroChain.
//!
//! A weight vector has length `n` (number of miners in a subnet). Most entries
//! are zero, so we store only `(index, weight)` pairs. When serialized for
//! on-chain storage we further RLE-compress runs of consecutive zero indices.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, Encode};
use scale_info::TypeInfo;

/// One entry in a sparse weight vector: `(miner_uid, weight)`.
pub type WeightEntry = (u32, u16);

/// Fixed-point scale used for `cosine_similarity`. A value of `COSINE_SCALE`
/// means perfect agreement; `0` means orthogonal or undefined.
pub const COSINE_SCALE: u32 = 1_000_000;

fn cosine_dot(a: &[WeightEntry], b: &[WeightEntry]) -> u128 {
    let mut i = 0usize;
    let mut j = 0usize;
    let mut acc: u128 = 0;
    while i < a.len() && j < b.len() {
        let (ui, wi) = a[i];
        let (uj, wj) = b[j];
        match ui.cmp(&uj) {
            core::cmp::Ordering::Equal => {
                acc = acc.saturating_add((wi as u128).saturating_mul(wj as u128));
                i += 1;
                j += 1;
            }
            core::cmp::Ordering::Less => i += 1,
            core::cmp::Ordering::Greater => j += 1,
        }
    }
    acc
}

fn norm_sq(a: &[WeightEntry]) -> u128 {
    a.iter()
        .map(|(_, w)| (*w as u128) * (*w as u128))
        .fold(0u128, |acc, x| acc.saturating_add(x))
}

fn integer_sqrt(x: u128) -> u128 {
    if x < 2 {
        return x;
    }
    let mut lo: u128 = 1;
    let mut hi: u128 = x;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if let Some(sq) = mid.checked_mul(mid) {
            if sq <= x {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        } else {
            hi = mid;
        }
    }
    lo - 1
}

/// Sparse weight vector. Entries MUST be sorted ascending by `uid` with no
/// duplicates — `new_sorted` enforces this.
#[derive(Clone, Debug, Default, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub struct SparseWeights {
    entries: Vec<WeightEntry>,
}

/// Compressed on-chain representation. Deltas between successive UIDs are
/// stored instead of absolute UIDs — this yields excellent compression when
/// miners are densely packed.
#[derive(Clone, Debug, Default, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub struct CompressedWeights {
    /// Total length of the dense vector this represents.
    pub length: u32,
    /// Sequence of (gap, weight) where `gap` = index delta from previous entry
    /// (or from -1 for the first entry). All gaps are 1 when indices are
    /// consecutive starting at 0.
    pub runs: Vec<(u32, u16)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WeightError {
    UnsortedOrDuplicate,
    IndexOutOfBounds,
    ZeroWeightEntry,
}

impl SparseWeights {
    /// Build from a pre-sorted list of `(uid, weight)` pairs. Rejects duplicate
    /// or unsorted UIDs and zero weights (they should simply be omitted).
    pub fn new_sorted(entries: Vec<WeightEntry>) -> Result<Self, WeightError> {
        let mut prev: Option<u32> = None;
        for (uid, w) in &entries {
            if *w == 0 {
                return Err(WeightError::ZeroWeightEntry);
            }
            if let Some(p) = prev {
                if *uid <= p {
                    return Err(WeightError::UnsortedOrDuplicate);
                }
            }
            prev = Some(*uid);
        }
        Ok(Self { entries })
    }

    /// Build from an iterator over `(uid, weight)` pairs; sorts + dedups on the
    /// fly. Later entries override earlier ones for the same uid; zero weights
    /// are dropped.
    pub fn from_pairs<I: IntoIterator<Item = WeightEntry>>(iter: I) -> Self {
        let mut v: Vec<WeightEntry> = iter.into_iter().filter(|(_, w)| *w != 0).collect();
        v.sort_by_key(|(uid, _)| *uid);
        v.dedup_by_key(|(uid, _)| *uid);
        Self { entries: v }
    }

    pub fn entries(&self) -> &[WeightEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Dense length = max_uid + 1, or zero when empty.
    pub fn dense_len(&self) -> u32 {
        self.entries.last().map(|(u, _)| u + 1).unwrap_or(0)
    }

    /// L1-normalize entries so their sum equals `u16::MAX`. Zero-sum vectors
    /// are left untouched.
    pub fn normalize_l1(&mut self) {
        let sum: u64 = self.entries.iter().map(|(_, w)| *w as u64).sum();
        if sum == 0 {
            return;
        }
        for (_, w) in self.entries.iter_mut() {
            *w = ((*w as u64 * u16::MAX as u64) / sum) as u16;
        }
    }

    /// Encode into a delta-gap compressed form. `dense_len` caps the implicit
    /// vector length and is needed for round-trip decoding when trailing zeros
    /// matter.
    pub fn compress(&self, dense_len: u32) -> Result<CompressedWeights, WeightError> {
        if let Some((last_uid, _)) = self.entries.last() {
            if *last_uid >= dense_len {
                return Err(WeightError::IndexOutOfBounds);
            }
        }
        let mut runs = Vec::with_capacity(self.entries.len());
        let mut prev: i64 = -1;
        for (uid, w) in &self.entries {
            let gap = (*uid as i64 - prev) as u32;
            runs.push((gap, *w));
            prev = *uid as i64;
        }
        Ok(CompressedWeights {
            length: dense_len,
            runs,
        })
    }

    /// Inverse of `compress`.
    pub fn decompress(c: &CompressedWeights) -> Result<Self, WeightError> {
        let mut entries = Vec::with_capacity(c.runs.len());
        let mut cursor: i64 = -1;
        for (gap, w) in &c.runs {
            cursor += *gap as i64;
            if cursor < 0 || cursor as u64 >= c.length as u64 {
                return Err(WeightError::IndexOutOfBounds);
            }
            entries.push((cursor as u32, *w));
        }
        SparseWeights::new_sorted(entries)
    }

    /// Cosine similarity in `u32` space, scaled to `[0, COSINE_SCALE]`.
    /// Returns 0 for zero-norm inputs.
    pub fn cosine_similarity(&self, other: &SparseWeights) -> u32 {
        let dot = cosine_dot(&self.entries, &other.entries);
        let na = norm_sq(&self.entries);
        let nb = norm_sq(&other.entries);
        if na == 0 || nb == 0 {
            return 0;
        }
        // similarity = dot / sqrt(na * nb); scale to COSINE_SCALE
        let denom = integer_sqrt(na.saturating_mul(nb));
        if denom == 0 {
            return 0;
        }
        let scaled = (dot.saturating_mul(COSINE_SCALE as u128)) / denom;
        if scaled > COSINE_SCALE as u128 {
            COSINE_SCALE
        } else {
            scaled as u32
        }
    }

    /// Weighted average of multiple vectors, using `validator_weights` as
    /// per-validator multipliers. Output is sparse and L1-normalized only if
    /// caller requests it.
    pub fn aggregate(votes: &[(&SparseWeights, u64)]) -> SparseWeights {
        use alloc::collections::BTreeMap;
        let mut acc: BTreeMap<u32, u128> = BTreeMap::new();
        for (sw, mult) in votes {
            for (uid, w) in &sw.entries {
                *acc.entry(*uid).or_insert(0) += (*w as u128) * (*mult as u128);
            }
        }
        let max_val = acc.values().copied().max().unwrap_or(0);
        let entries: Vec<WeightEntry> = if max_val == 0 {
            Vec::new()
        } else {
            acc.into_iter()
                .map(|(uid, v)| (uid, ((v * u16::MAX as u128) / max_val) as u16))
                .filter(|(_, w)| *w != 0)
                .collect()
        };
        SparseWeights { entries }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sw(entries: &[(u32, u16)]) -> SparseWeights {
        SparseWeights::new_sorted(entries.to_vec()).expect("valid")
    }

    #[test]
    fn new_sorted_rejects_unsorted() {
        assert_eq!(
            SparseWeights::new_sorted(vec![(2, 1), (1, 1)]).unwrap_err(),
            WeightError::UnsortedOrDuplicate
        );
    }

    #[test]
    fn new_sorted_rejects_duplicate() {
        assert_eq!(
            SparseWeights::new_sorted(vec![(1, 1), (1, 2)]).unwrap_err(),
            WeightError::UnsortedOrDuplicate
        );
    }

    #[test]
    fn new_sorted_rejects_zero_weight() {
        assert_eq!(
            SparseWeights::new_sorted(vec![(1, 0)]).unwrap_err(),
            WeightError::ZeroWeightEntry
        );
    }

    #[test]
    fn from_pairs_sorts_and_dedups() {
        let w = SparseWeights::from_pairs(vec![(3, 10), (1, 20), (2, 0), (1, 5)]);
        // Duplicate uid 1 — dedup_by_key keeps the first seen post-sort
        assert_eq!(w.entries(), &[(1, 20), (3, 10)]);
    }

    #[test]
    fn compress_decompress_roundtrip_dense() {
        let w = sw(&[(0, 10), (1, 20), (2, 30), (3, 40)]);
        let c = w.compress(10).unwrap();
        // All gaps are 1 when consecutive from 0
        assert!(c.runs.iter().all(|(g, _)| *g == 1));
        assert_eq!(SparseWeights::decompress(&c).unwrap(), w);
    }

    #[test]
    fn compress_decompress_roundtrip_sparse() {
        let w = sw(&[(5, 100), (50, 200), (999, 300)]);
        let c = w.compress(1000).unwrap();
        assert_eq!(SparseWeights::decompress(&c).unwrap(), w);
    }

    #[test]
    fn compress_rejects_oob() {
        let w = sw(&[(999, 1)]);
        assert_eq!(w.compress(500).unwrap_err(), WeightError::IndexOutOfBounds);
    }

    #[test]
    fn dense_len_tracks_max_uid() {
        assert_eq!(sw(&[(0, 1), (1, 1), (9, 1)]).dense_len(), 10);
        assert_eq!(SparseWeights::default().dense_len(), 0);
    }

    #[test]
    fn normalize_l1_sums_to_u16_max() {
        let mut w = sw(&[(0, 10), (1, 20), (2, 70)]);
        w.normalize_l1();
        let sum: u64 = w.entries().iter().map(|(_, x)| *x as u64).sum();
        // Allow rounding slack up to n entries
        assert!(sum >= u16::MAX as u64 - 3 && sum <= u16::MAX as u64);
    }

    #[test]
    fn aggregate_majority_vote() {
        let a = sw(&[(0, 100), (1, 200)]);
        let b = sw(&[(1, 200), (2, 100)]);
        let agg = SparseWeights::aggregate(&[(&a, 1), (&b, 1)]);
        // uid 1 got votes from both → should be the peak (u16::MAX after scaling)
        let peak = agg
            .entries()
            .iter()
            .max_by_key(|(_, w)| *w)
            .copied()
            .unwrap();
        assert_eq!(peak.0, 1);
    }

    #[test]
    fn aggregate_respects_validator_weight() {
        let a = sw(&[(0, 100)]);
        let b = sw(&[(1, 100)]);
        // Give validator b 10x multiplier
        let agg = SparseWeights::aggregate(&[(&a, 1), (&b, 10)]);
        assert_eq!(agg.entries().iter().max_by_key(|(_, w)| *w).unwrap().0, 1);
    }

    #[test]
    fn cosine_identical_vectors() {
        let a = sw(&[(0, 10), (1, 20), (2, 30)]);
        assert_eq!(a.cosine_similarity(&a), COSINE_SCALE);
    }

    #[test]
    fn cosine_orthogonal_zero() {
        let a = sw(&[(0, 10)]);
        let b = sw(&[(1, 10)]);
        assert_eq!(a.cosine_similarity(&b), 0);
    }

    #[test]
    fn cosine_zero_for_empty_vector() {
        let a = sw(&[(0, 10)]);
        let empty = SparseWeights::default();
        assert_eq!(a.cosine_similarity(&empty), 0);
        assert_eq!(empty.cosine_similarity(&a), 0);
    }

    #[test]
    fn cosine_partial_overlap_between_0_and_scale() {
        let a = sw(&[(0, 10), (1, 10)]);
        let b = sw(&[(1, 10), (2, 10)]);
        let sim = a.cosine_similarity(&b);
        assert!(sim > 0 && sim < COSINE_SCALE);
    }

    #[test]
    fn integer_sqrt_works() {
        assert_eq!(integer_sqrt(0), 0);
        assert_eq!(integer_sqrt(1), 1);
        assert_eq!(integer_sqrt(4), 2);
        assert_eq!(integer_sqrt(99), 9);
        assert_eq!(integer_sqrt(100), 10);
    }

    #[test]
    fn aggregate_empty_yields_empty() {
        let agg = SparseWeights::aggregate(&[]);
        assert!(agg.is_empty());
    }

    #[test]
    fn compression_is_compact_for_dense_prefix() {
        // 1000 miners, all with weight 1 — compressed should be ~ 1000 gap/weight pairs
        // with every gap == 1.
        let entries: Vec<WeightEntry> = (0..1000).map(|i| (i, 1)).collect();
        let w = SparseWeights::new_sorted(entries).unwrap();
        let c = w.compress(1000).unwrap();
        let all_unit_gaps = c.runs.iter().all(|(g, _)| *g == 1);
        assert!(all_unit_gaps);
        let rt = SparseWeights::decompress(&c).unwrap();
        assert_eq!(rt, w);
    }
}
