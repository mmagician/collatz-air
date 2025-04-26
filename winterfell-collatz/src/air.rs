use winterfell::{
    math::{fields::f128::BaseElement, FieldElement},
    Air, AirContext, Assertion, EvaluationFrame, TransitionConstraintDegree,
};
use crate::utils::PublicInputs;
use crate::utils::is_binary;

/// AIR for proving Collatz conjecture sequences.
/// The trace consists of N columns, each representing a bit in the binary representation (LSB first)
/// of the current number in the sequence.
pub struct CollatzAir<const N: usize> {
    context: AirContext<BaseElement>,
    first: [BaseElement; N],
}

impl<const N: usize> Air for CollatzAir<N> {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs<N>;

    fn new(
        trace_info: winterfell::TraceInfo,
        pub_inputs: Self::PublicInputs,
        options: winterfell::ProofOptions,
    ) -> Self {
        assert_eq!(N, trace_info.width());
        // We have N consistency constraints(part of the transition constraints): the value in each column must be binary (degree 2: col*col - col = 0)
        let mut transition_constraints = vec![TransitionConstraintDegree::new(2); N];

        // Main transition constraint, multiplies the first column (parity bit) by the weighted sum of the other columns, resulting in degree 2 constraint. Then we apply the OR condition (degree 1), resulting in total degree 3.
        transition_constraints.push(TransitionConstraintDegree::new(3));

        // We have 2*N boundary constraints: N for the first row (must equal the public input), N for the last row (must be 1)
        let num_boundary_constraints = 2 * N;

        CollatzAir {
            context: AirContext::new(trace_info, transition_constraints, num_boundary_constraints, options),
            first: pub_inputs.values,
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

        // Consistency constraint: ensure each cell is binary
        for i in 0..N {
            result[i] = is_binary(next[i]);
        }

        let current_weighted_sum =
            (0..N).fold(E::ZERO, |acc, i| acc + (E::from(2u32.pow(i as u32)) * current[i]));
        let next_weighted_sum =
            (0..N).fold(E::ZERO, |acc, i| acc + (E::from(2u32.pow(i as u32)) * next[i]));

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
        result[N] =
            // repeat the current row, OR
            (next_weighted_sum - current_weighted_sum) 
            // apply the Collatz transition rule
            * (
                (E::from(2u32) * next_weighted_sum)
                - (current[0] * E::from(2u32) * (current_weighted_sum * E::from(3u32) + E::ONE)
                    + (E::ONE - current[0]) * current_weighted_sum)
        );
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // Boundary constraint: the whole first row is the initial state
        let mut assertions: Vec<Assertion<BaseElement>> =
            (0..N).map(|i| Assertion::single(i, 0, self.first[i])).collect();

        // Boundary constraint: the weighted sum of the last row is 1, i.e. the first column is 1, the rest are 0
        let last_step = self.trace_length() - 1;
        assertions.push(Assertion::single(0, last_step, Self::BaseField::ONE));
        for i in 1..N {
            assertions.push(Assertion::single(i, last_step, Self::BaseField::ZERO));
        }

        assertions
    }
}
