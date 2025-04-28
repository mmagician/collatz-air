use crate::utils::is_binary;
use crate::utils::PublicInputs;
use winterfell::{
    math::{fields::f128::BaseElement, FieldElement},
    Air, AirContext, Assertion, EvaluationFrame, TransitionConstraintDegree,
};

/// AIR for proving Collatz conjecture sequences.
/// The trace consists of N columns, each representing a bit in the binary representation (LSB first)
/// of the current number in the sequence, plus two additional columns:
/// - Column N: step counter
/// - Column N+1: transition flag (1 = transition, 0 = repeat)
pub struct CollatzAir<const N: usize> {
    context: AirContext<BaseElement>,
    first: [BaseElement; N],
    steps_count: BaseElement,
}

impl<const N: usize> Air for CollatzAir<N> {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs<N>;

    fn new(
        trace_info: winterfell::TraceInfo,
        pub_inputs: Self::PublicInputs,
        options: winterfell::ProofOptions,
    ) -> Self {
        assert_eq!(N + 2, trace_info.width());
        // We have N consistency constraints for binary values, plus 1 for the transition flag
        let mut transition_constraints = vec![TransitionConstraintDegree::new(2); N + 1];

        // Main transition constraint, multiplies the `is_transition` column (degree 1) by the weighted sum of the other columns (degree 1) and the parity bit (first column, degree 1), resulting in degree 3 constraint.
        transition_constraints.push(TransitionConstraintDegree::new(3));
        // Step counter constraint (degree 2)
        transition_constraints.push(TransitionConstraintDegree::new(2));

        // We have 2*N boundary constraints for values, + 1 for initial step counter, + 1 for final step counter, + 1 for the initial transition flag
        let num_boundary_constraints = 2 * N + 3;

        CollatzAir {
            context: AirContext::new(
                trace_info,
                transition_constraints,
                num_boundary_constraints,
                options,
            ),
            first: pub_inputs.values,
            steps_count: pub_inputs.steps_count,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        _periodic_values: &[E],
        result: &mut [E],
    ) {
        let current = frame.current();
        let next = frame.next();

        let step_counter = current[N];
        let next_step_counter = next[N];
        let next_is_transition = next[N + 1];

        // Consistency constraint: ensure each cell in the binary decomposition column is indeed a bit.
        for i in 0..N {
            result[i] = is_binary(next[i]);
        }

        // Ensure transition flag is binary
        result[N] = is_binary(next[N + 1]);

        let current_weighted_sum = (0..N).fold(E::ZERO, |acc, i| {
            acc + (E::from(2u32.pow(i as u32)) * current[i])
        });
        let next_weighted_sum = (0..N).fold(E::ZERO, |acc, i| {
            acc + (E::from(2u32.pow(i as u32)) * next[i])
        });

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
        result[N + 1] =
            // Apply the Collatz transition rule
            next_is_transition * (
                (E::from(2u32) * next_weighted_sum)
                - (current[0] * E::from(2u32) * (current_weighted_sum * E::from(3u32) + E::ONE)
                + (E::ONE - current[0]) * current_weighted_sum)
            )
            // No transition, repeat the current row
            - (E::ONE - next_is_transition) * (next_weighted_sum - current_weighted_sum);

        // Step counter constraint:
        // If next_is_transition = 1, increment step counter
        // If next_is_transition = 0, keep step counter the same
        result[N + 2] = next_is_transition * (next_step_counter - step_counter - E::ONE)
            - (E::ONE - next_is_transition) * (next_step_counter - step_counter);
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // Boundary constraint: the whole first row is the initial state
        let mut assertions: Vec<Assertion<BaseElement>> = (0..N)
            .map(|i| Assertion::single(i, 0, self.first[i]))
            .collect();

        // Initial step counter is 0
        assertions.push(Assertion::single(N, 0, BaseElement::ZERO));
        // Initial transition flag is 0 (not a transition)
        assertions.push(Assertion::single(N + 1, 0, BaseElement::ZERO));

        // Boundary constraint: the weighted sum of the last row is 1, i.e. the first column is 1, the rest are 0
        let last_step = self.trace_length() - 1;
        assertions.push(Assertion::single(0, last_step, Self::BaseField::ONE));
        for i in 1..N {
            assertions.push(Assertion::single(i, last_step, Self::BaseField::ZERO));
        }

        // The last row's step counter should match the expected steps_count
        assertions.push(Assertion::single(N, last_step, self.steps_count));

        // We don't have an explicit ending boundary constraint for the last row's is_transition flag:
        // if the trace_length perfectly matches the steps_count without padding, then it's a transition row, otherwise it's not.

        assertions
    }
}
