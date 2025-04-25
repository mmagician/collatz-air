use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::Field;
use p3_field::PrimeCharacteristicRing;
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;

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

        // Enforce starting values based on the binary representation of starting_value
        for i in 0..N {
            builder.when_first_row().assert_eq(
                local[i],
                AB::Expr::from_bool((self.starting_value >> i & 1) == 1),
            );
        }

        // Enforce boolean values in each cell
        for i in 0..N {
            builder.when_transition().assert_bool(local[i]);
        }

        // Calculate the current value and next value from their binary representations
        let current_weighted_sum = (0..N).fold(AB::Expr::ZERO, |acc, i| {
            acc + (AB::Expr::from_u32(2u32.pow(i as u32)) * local[i])
        });

        let next_weighted_sum = (0..N).fold(AB::Expr::ZERO, |acc, i| {
            acc + (AB::Expr::from_u32(2u32.pow(i as u32)) * next[i])
        });

        let is_odd = local[0].clone();

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

        // Constrain the final value to be 1 (Collatz conjecture's end condition)
        builder.when_last_row().assert_eq(local[0], AB::Expr::ONE);
        for i in 1..N {
            builder.when_last_row().assert_eq(local[i], AB::Expr::ZERO);
        }
    }
}

pub fn generate_collatz_trace<const N: usize, F: Field>(starting_value: u32) -> RowMajorMatrix<F> {
    let mut sequence = utils::compute_collatz_sequence(starting_value);
    sequence.resize((sequence.len()).next_power_of_two(), 1);
    let mut values = Vec::with_capacity(N * sequence.len());
    for i in 0..sequence.len() {
        for j in 0..N {
            values.push(F::from_u32(sequence[i] >> j & 1));
        }
    }
    RowMajorMatrix::new(values, N)
}

// --------------------------------------------------------------------------------------------
// UTILS
// --------------------------------------------------------------------------------------------
pub mod utils {

    pub fn compute_collatz_sequence(n: u32) -> Vec<u32> {
        let mut sequence = Vec::new();
        let mut current = n;

        while current != 1 {
            sequence.push(current);
            if current % 2 == 0 {
                current = current / 2;
            } else {
                current = 3 * current + 1;
            }
        }
        sequence.push(1);
        sequence
    }
}
