use winterfell::{
    math::{fields::f128::BaseElement, FieldElement, ToElements},
    Air, AirContext, Assertion, EvaluationFrame, TransitionConstraintDegree,
};

pub struct CollatzAir<const N: usize> {
    context: AirContext<BaseElement>,
    first: [BaseElement; N],
}

// The public inputs is required to implement the `ToElements` trait.
// Due to the orphan rule, we need to create a newtype to hold the array.
pub struct PublicInputs<const N: usize> ([BaseElement; N]);

impl<const N: usize> PublicInputs<N> {
    pub fn new(value: [BaseElement; N]) -> Self {
        Self(value)
    }
}

impl<const N: usize> From<u32> for PublicInputs<N> {
    fn from(value: u32) -> Self {
        let mut first = [BaseElement::ZERO; N];
        for i in 0..N {
            first[i] = BaseElement::from((value >> i) & 1);
        }
        PublicInputs(first)
    }
}

impl<const N: usize> ToElements<BaseElement> for PublicInputs<N> {
    fn to_elements(&self) -> Vec<BaseElement> {
        self.0.to_vec()
    }
}

/// Returns zero only when a = zero || a == one.
fn is_binary<E: FieldElement>(a: E) -> E {
    a * a - a
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
        // binary transition constraints for each column (col*col - col) have degree 2
        let mut transition_constraints = vec![TransitionConstraintDegree::new(2); N];
        // main transition constraint, multiplies the first column (partiy bit) by the weighted sum of the other columns, resulting in degree 2 constraint. Then we apply the OR condition (degree 1), resulting in total degree 3.
        transition_constraints.push(TransitionConstraintDegree::new(3));

        // we have 2*N boundary constraints: N for the first row (must equal the public input), N for the last row (must be 1)
        CollatzAir {
            context: AirContext::new(trace_info, transition_constraints, 2 * N, options),
            first: pub_inputs.0,
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

        debug_assert_eq!(N, current.len());
        debug_assert_eq!(N, next.len());
        // ensure each cell is binary
        for i in 0..N {
            result[i] = is_binary(next[i]);
        }

        let current_weighted_sum =
            (0..N).fold(E::ZERO, |acc, i| acc + (E::from(2u32.pow(i as u32)) * current[i]));
        let next_weighted_sum =
            (0..N).fold(E::ZERO, |acc, i| acc + (E::from(2u32.pow(i as u32)) * next[i]));

        // next transitions constraint: apply the collatz_rule OR repeat row
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
        // the whole first row is the initial state
        let mut assertions: Vec<Assertion<BaseElement>> =
            (0..N).map(|i| Assertion::single(i, 0, self.first[i])).collect();

        // the weighted sum of the last row is 1
        // first column is 1
        let last_step = self.trace_length() - 1;
        // the rest are 0
        assertions.push(Assertion::single(0, last_step, Self::BaseField::ONE));
        for i in 1..N {
            assertions.push(Assertion::single(i, last_step, Self::BaseField::ZERO));
        }

        assertions
    }
}
