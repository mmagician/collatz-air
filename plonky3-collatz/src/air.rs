use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::Field;
use p3_field::PrimeCharacteristicRing;
use p3_matrix::Matrix;

/// AIR for proving Collatz conjecture sequences.
/// The trace consists of N columns, each representing a bit in the binary representation (LSB first)
/// of the current number in the sequence.
pub struct CollatzAir<const N: usize> {
    pub starting_value: u32,
}

impl<const N: usize, F: Field> BaseAir<F> for CollatzAir<N> {
    fn width(&self) -> usize {
        N
    }
}

impl<AB: AirBuilder, const N: usize> Air<AB> for CollatzAir<N> {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0).expect("The matrix is empty?");
        let next = main.row_slice(1).expect("The matrix only has 1 row?");

        // Boundary constraint: enforce starting values based on the binary representation of starting_value
        for i in 0..N {
            builder.when_first_row().assert_eq(
                local[i],
                AB::Expr::from_bool((self.starting_value >> i & 1) == 1),
            );
        }

        // Consistency constraint: ensure each cell is binary
        for i in 0..N {
            builder.when_transition().assert_bool(local[i]);
        }

        let current_weighted_sum = (0..N).fold(AB::Expr::ZERO, |acc, i| {
            acc + (AB::Expr::from_u32(2u32.pow(i as u32)) * local[i])
        });

        let next_weighted_sum = (0..N).fold(AB::Expr::ZERO, |acc, i| {
            acc + (AB::Expr::from_u32(2u32.pow(i as u32)) * next[i])
        });

        let is_odd = local[0].clone();

        // Main transition constraint: apply the collatz_rule OR repeat row
        // (Needed to ensure valid transitions for the entire trace length, even when we pad with 1's to the next power of two).
        // Note, that while our prover fills the remainder of the trace with 1's, it actually doesn't matter *which* row is repeated.
        // E.g. For the Collatz sequence "4, 2, 1", the prover could fill the trace with (the binary representations of):
        // [4, 4, 2, 1], or
        // [4, 2, 2, 1], or
        // [4, 2, 1, 1],
        // and all should be accepted.

        // Collatz transition rule:
        // next_weighted_sum = 
        //      is_odd * (current_weighted_sum * 3 + 1) + 
        //      (1 - is_odd) * (current_weighted_sum / 2)
        // 
        // Note that since we can't have division, we multiply all terms by 2, resulting in:
        // 2 * next_weighted_sum = 
        //      is_odd * 2 * (current_weighted_sum * 3 + 1) + 
        //      (1 - is_odd) * current_weighted_sum
        builder.when_transition().assert_zero(
            // repeat the current row, OR
            (next_weighted_sum.clone() - current_weighted_sum.clone())
                // apply the Collatz transition rule
            * (
                 (AB::Expr::TWO * next_weighted_sum) 
                    - (is_odd * AB::Expr::TWO * (current_weighted_sum.clone() * AB::Expr::from_u32(3) + AB::Expr::ONE)
                    + (AB::Expr::ONE - is_odd) * current_weighted_sum)
                ),
        );

        // Boundary constraint: the weighted sum of the last row is 1, i.e. the first column is 1, the rest are 0
        builder.when_last_row().assert_eq(local[0], AB::Expr::ONE);
        for i in 1..N {
            builder.when_last_row().assert_eq(local[i], AB::Expr::ZERO);
        }
    }
}
