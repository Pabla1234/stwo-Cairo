use itertools::{chain, Itertools};
use prover_types::simd::LOG_N_LANES;
use stwo_prover::constraint_framework::preprocessed_columns::{IsFirst, PreProcessedColumnId};
use stwo_prover::core::backend::simd::m31::{PackedM31, N_LANES};
use stwo_prover::core::backend::simd::SimdBackend;
use stwo_prover::core::backend::Col;
use stwo_prover::core::fields::m31::{BaseField, M31};
use stwo_prover::core::poly::circle::{CanonicCoset, CircleEvaluation};
use stwo_prover::core::poly::BitReversedOrder;

use super::LOG_MAX_ROWS;
use crate::components::range_check_vector::SIMD_ENUMERATION_0;

const N_PREPROCESSED_COLUMN_SIZES: usize = (LOG_MAX_ROWS - LOG_N_LANES) as usize + 1;

// List of sizes to initialize the preprocessed trace with for `PreprocessedColumn::IsFirst`.
const IS_FIRST_LOG_SIZES: [u32; N_PREPROCESSED_COLUMN_SIZES] = preprocessed_log_sizes();

// List of sizes to initialize the preprocessed trace with for `PreprocessedColumn::Seq`.
const SEQ_LOG_SIZES: [u32; N_PREPROCESSED_COLUMN_SIZES] = preprocessed_log_sizes();

/// [LOG_MAX_ROWS, LOG_MAX_ROWS - 1, ..., LOG_N_LANES]
const fn preprocessed_log_sizes() -> [u32; N_PREPROCESSED_COLUMN_SIZES] {
    let mut arr = [0; N_PREPROCESSED_COLUMN_SIZES];
    let mut i = 0;
    while i < N_PREPROCESSED_COLUMN_SIZES {
        arr[i] = LOG_MAX_ROWS - i as u32;
        i += 1;
    }
    arr
}

pub enum PreProcessedColumn {
    IsFirst(IsFirst),
    Seq(Seq),
}
impl PreProcessedColumn {
    pub fn log_size(&self) -> u32 {
        match self {
            PreProcessedColumn::IsFirst(column) => column.log_size,
            PreProcessedColumn::Seq(column) => column.log_size,
        }
    }
    pub fn id(&self) -> PreProcessedColumnId {
        match self {
            PreProcessedColumn::IsFirst(column) => column.id(),
            PreProcessedColumn::Seq(column) => column.id(),
        }
    }
    pub fn gen_column_simd(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder> {
        match self {
            PreProcessedColumn::IsFirst(column) => column.gen_column_simd(),
            PreProcessedColumn::Seq(column) => column.gen_column_simd(),
        }
    }

    pub fn packed_at(&self, vec_row: usize) -> PackedM31 {
        match self {
            PreProcessedColumn::IsFirst(column) => column.packed_at(vec_row),
            PreProcessedColumn::Seq(column) => column.packed_at(vec_row),
        }
    }
}

/// Returns column info for the preprocessed trace.
pub fn preprocessed_trace_columns() -> Vec<PreProcessedColumn> {
    let is_first_columns =
        IS_FIRST_LOG_SIZES.map(|log_size| PreProcessedColumn::IsFirst(IsFirst::new(log_size)));
    let seq_columns = SEQ_LOG_SIZES.map(|log_size| PreProcessedColumn::Seq(Seq::new(log_size)));
    chain![is_first_columns, seq_columns]
        .sorted_by_key(|column| std::cmp::Reverse(column.log_size()))
        .collect_vec()
}

/// A column with the numbers [0..(2^log_size)-1].
#[derive(Debug, Clone)]
pub struct Seq {
    pub log_size: u32,
}
impl Seq {
    pub const fn new(log_size: u32) -> Self {
        Self { log_size }
    }

    pub fn packed_at(&self, vec_row: usize) -> PackedM31 {
        assert!(vec_row < (1 << self.log_size) / N_LANES);
        PackedM31::broadcast(M31::from(vec_row * N_LANES))
            + unsafe { PackedM31::from_simd_unchecked(SIMD_ENUMERATION_0) }
    }

    pub fn gen_column_simd(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder> {
        let col = Col::<SimdBackend, BaseField>::from_iter(
            (0..(1 << self.log_size)).map(BaseField::from),
        );
        CircleEvaluation::new(CanonicCoset::new(self.log_size).circle_domain(), col)
    }

    pub fn id(&self) -> PreProcessedColumnId {
        PreProcessedColumnId {
            id: format!("preprocessed_seq_{}", self.log_size).to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_columns_are_in_decending_order() {
        let columns = preprocessed_trace_columns();

        assert!(columns
            .windows(2)
            .all(|w| w[0].log_size() >= w[1].log_size()));
    }
}
