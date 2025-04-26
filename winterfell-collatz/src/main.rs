mod air;
mod prover;
mod utils;

use air::*;
use prover::*;

use tracing::level_filters::LevelFilter;
use tracing_forest::ForestLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use utils::compute_collatz_sequence;
use winterfell::{
    crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree},
    math::fields::f128::BaseElement,
    verify, BatchingMethod, FieldExtension, ProofOptions, Prover,
};
const N: usize = 6;

fn main() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    let starting_value = 52;
    let sequence = compute_collatz_sequence(starting_value);
    let max_element = sequence.iter().max().unwrap_or(&0);
    let max_bits_in_sequence = 32 - max_element.leading_zeros() as usize;

    assert_eq!(max_bits_in_sequence, N, "The number of trace columns must match the number of bits in the max element of the sequence");

    let proof_options = ProofOptions::new(
        28,
        8,
        0,
        FieldExtension::Quadratic,
        4,
        7,
        BatchingMethod::Linear,
        BatchingMethod::Linear,
    );

    let prover =
        CollatzProver::<Blake3_256<BaseElement>, N>::new(proof_options.clone(), starting_value);

    let trace = prover.build_trace();
    let public_inputs = prover.get_pub_inputs(&trace);
    let proof = prover.prove(trace).unwrap();

    let acceptable_options = winterfell::AcceptableOptions::OptionSet(vec![proof_options]);
    assert!(verify::<
        CollatzAir<N>,
        Blake3_256<BaseElement>,
        DefaultRandomCoin<Blake3_256<BaseElement>>,
        MerkleTree<Blake3_256<BaseElement>>,
    >(proof, public_inputs, &acceptable_options)
    .is_ok());
}
