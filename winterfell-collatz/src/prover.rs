use crate::utils::compute_collatz_sequence;
use std::marker::PhantomData;
use winterfell::crypto::{DefaultRandomCoin, ElementHasher, MerkleTree};
use winterfell::math::fields::f128::BaseElement;
use winterfell::math::FieldElement;
use winterfell::matrix::ColMatrix;
use winterfell::{
    AuxRandElements, CompositionPoly, CompositionPolyTrace, ConstraintCompositionCoefficients,
    DefaultConstraintCommitment, DefaultConstraintEvaluator, DefaultTraceLde, PartitionOptions,
    ProofOptions, Prover, StarkDomain, TraceInfo, TracePolyTable, TraceTable,
};

use crate::air::{CollatzAir, PublicInputs};

pub struct CollatzProver<H: ElementHasher, const N: usize> {
    options: ProofOptions,
    starting_value: u32,
    _hasher: PhantomData<H>,
}

impl<H: ElementHasher, const N: usize> CollatzProver<H, N> {
    pub fn new(options: ProofOptions, starting_value: u32) -> Self {
        Self {
            options,
            starting_value,
            _hasher: PhantomData,
        }
    }

    pub fn build_trace(&self) -> TraceTable<BaseElement> {
        // we need to dynamically compute the trace length, it depends on the instance starting value
        let mut sequence = compute_collatz_sequence(self.starting_value);
        let mut trace_length = sequence.len();
        // pad the trace length to the next power of 2
        trace_length = trace_length.next_power_of_two();
        // fill the rest of the sequence with ones
        sequence.resize(trace_length, 1);

        let mut trace = TraceTable::new(N, trace_length);
        trace.fill(
            |state| {
                for i in 0..N {
                    state[i] = BaseElement::from((self.starting_value >> i) & 1);
                }
            },
            |j, state| {
                let next_val = sequence[j + 1];

                for i in 0..N {
                    state[i] = BaseElement::from((next_val >> i) & 1);
                }
            },
        );
        trace
    }
}

impl<H: ElementHasher, const N: usize> Prover for CollatzProver<H, N>
where
    H: ElementHasher<BaseField = BaseElement> + Sync,
{
    type BaseField = BaseElement;
    type Air = CollatzAir<N>;
    type Trace = TraceTable<BaseElement>;
    type HashFn = H;
    type VC = MerkleTree<H>;
    type RandomCoin = DefaultRandomCoin<Self::HashFn>;
    type TraceLde<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultTraceLde<E, Self::HashFn, Self::VC>;
    type ConstraintCommitment<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintCommitment<E, H, Self::VC>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintEvaluator<'a, Self::Air, E>;

    fn get_pub_inputs(
        &self,
        trace: &Self::Trace,
    ) -> <<Self as Prover>::Air as winterfell::Air>::PublicInputs {
        // public input is the first step of the sequence
        let mut first = [BaseElement::ZERO; N];
        trace.read_row_into(0, &mut first);

        PublicInputs::new(first)
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }

    fn new_trace_lde<E>(
        &self,
        trace_info: &TraceInfo,
        main_trace: &ColMatrix<Self::BaseField>,
        domain: &StarkDomain<Self::BaseField>,
        partition_option: PartitionOptions,
    ) -> (Self::TraceLde<E>, TracePolyTable<E>)
    where
        E: FieldElement<BaseField = Self::BaseField>,
    {
        DefaultTraceLde::new(trace_info, main_trace, domain, partition_option)
    }

    fn new_evaluator<'a, E>(
        &self,
        air: &'a Self::Air,
        aux_rand_elements: Option<AuxRandElements<E>>,
        composition_coefficients: ConstraintCompositionCoefficients<E>,
    ) -> Self::ConstraintEvaluator<'a, E>
    where
        E: FieldElement<BaseField = Self::BaseField>,
    {
        DefaultConstraintEvaluator::new(air, aux_rand_elements, composition_coefficients)
    }

    fn build_constraint_commitment<E>(
        &self,
        composition_poly_trace: CompositionPolyTrace<E>,
        num_constraint_composition_columns: usize,
        domain: &StarkDomain<Self::BaseField>,
        partition_options: PartitionOptions,
    ) -> (Self::ConstraintCommitment<E>, CompositionPoly<E>)
    where
        E: FieldElement<BaseField = Self::BaseField>,
    {
        DefaultConstraintCommitment::new(
            composition_poly_trace,
            num_constraint_composition_columns,
            domain,
            partition_options,
        )
    }
}
