use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::Field;
use p3_field::PrimeCharacteristicRing;
use p3_matrix::Matrix;

/// AIR for proving Collatz conjecture sequences.
/// The trace consists of N columns, each representing a bit in the binary representation (LSB first)
/// of the current number in the sequence, plus an additional column for the step counter.
pub struct CollatzAir<const N: usize> {
    pub starting_value: u32,
    pub steps_count: u32,
}

impl<const N: usize, F: Field> BaseAir<F> for CollatzAir<N> {
    fn width(&self) -> usize {
        // Add 1 for the step counter column
        N + 2
    }
}

impl<AB: AirBuilder, const N: usize> Air<AB> for CollatzAir<N> {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0).expect("The matrix is empty?");
        let next = main.row_slice(1).expect("The matrix only has 1 row?");

        let value_bits = &local[0..N];
        let next_value_bits = &next[0..N];
        let step_counter = local[N];
        let next_step_counter = next[N];
        let is_transition = local[N + 1];
        let next_is_transition = next[N + 1];

        // ------------------------------------------------------------------------------------------------
        // Initial boundary constraints
        // ------------------------------------------------------------------------------------------------

        // Boundary constraint: enforce starting values based on the binary representation of starting_value
        for i in 0..N {
            builder.when_first_row().assert_eq(
                value_bits[i],
                AB::Expr::from_bool((self.starting_value >> i & 1) == 1),
            );
        }

        // Initial step counter is 0
        builder
            .when_first_row()
            .assert_eq(step_counter, AB::Expr::ZERO);
        // The first row is not a transition row
        builder
            .when_first_row()
            .assert_eq(is_transition, AB::Expr::ZERO);

        // ------------------------------------------------------------------------------------------------
        // Transition constraints
        // ------------------------------------------------------------------------------------------------

        // Consistency constraint: ensure each cell in the binary decomposition column is indeed a bit.
        // Note, that we constrain the next row's value bits
        // (the first row is already guaranteed to be binary and correct due to the boundary constraint check)
        for i in 0..N {
            builder.when_transition().assert_bool(next_value_bits[i]);
        }
        // Consistency constraint: ensure the next `is_transition` value is indeed a bit.
        // Again, we constrain the next row's `is_transition`.
        // (the value in the first row is already guaranteed to be a zero by the boundary constraint)
        builder
            .when_transition()
            .assert_bool(next_is_transition.clone());

        let current_weighted_sum = (0..N).fold(AB::Expr::ZERO, |acc, i| {
            acc + (AB::Expr::from_u32(2u32.pow(i as u32)) * value_bits[i])
        });

        let next_weighted_sum = (0..N).fold(AB::Expr::ZERO, |acc, i| {
            acc + (AB::Expr::from_u32(2u32.pow(i as u32)) * next_value_bits[i])
        });

        let is_odd = value_bits[0].clone();

        // Main transition constraint: apply the collatz_rule OR repeat row
        builder.when_transition().assert_eq(
            // Apply the Collatz transition rule
            next_is_transition.clone()
                * ((AB::Expr::TWO * next_weighted_sum.clone())
                    - (is_odd
                        * AB::Expr::TWO
                        * (current_weighted_sum.clone() * AB::Expr::from_u32(3) + AB::Expr::ONE)
                        + (AB::Expr::ONE - is_odd) * current_weighted_sum.clone())),
            // No transition, repeat the current row
            (AB::Expr::ONE - next_is_transition.clone())
                * (current_weighted_sum - next_weighted_sum),
        );

        // Step counter constraint: If `is_transition` is 1, then `next_step_counter` should be incremented from one row to the next.
        // If no transition is made, then the step counter should be the same as the current step counter.
        builder.when_transition().assert_eq(
            // If there is a transition, then the step counter should be incremented
            next_is_transition.clone()
                * (next_step_counter.clone() - step_counter.clone() - AB::Expr::ONE),
            // If there is no transition, then the step counter should be the same
            (AB::Expr::ONE - next_is_transition.clone()) * (step_counter - next_step_counter),
        );

        // ------------------------------------------------------------------------------------------------
        // Ending boundary constraints
        // ------------------------------------------------------------------------------------------------

        // Boundary constraint: the weighted sum of the last row is 1, i.e. the first column is 1, the rest (till N) are 0
        builder
            .when_last_row()
            .assert_eq(value_bits[0], AB::Expr::ONE);
        for i in 1..N {
            builder
                .when_last_row()
                .assert_eq(value_bits[i], AB::Expr::ZERO);
        }

        // The last row's step counter should match the expected steps_count
        builder
            .when_last_row()
            .assert_eq(step_counter, AB::Expr::from_u32(self.steps_count));
        // We don't have an explicit ending boundary constraint for the last row's is_transition flag:
        // if the trace_length perfectly matches the steps_count without padding, then it's a transition row, otherwise it's not.
    }
}
