use std::fmt::Debug;

use p3_baby_bear::BabyBear;
use p3_challenger::{HashChallenger, SerializingChallenger32};
use p3_commit::ExtensionMmcs;
use p3_field::extension::BinomialExtensionField;
use p3_fri::{create_benchmark_fri_config, TwoAdicFriPcs};
use p3_keccak::Keccak256Hash;
use p3_merkle_tree::MerkleTreeMmcs;
use p3_symmetric::{CompressionFunctionFromHasher, SerializingHasher};
use p3_uni_stark::{prove, verify, StarkConfig};
use tracing_forest::util::LevelFilter;
use tracing_forest::ForestLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

mod air;
mod utils;

use air::*;
use utils::generate_collatz_trace;

const N: usize = 6;

// Type definitions
type Val = BabyBear;
type Challenge = BinomialExtensionField<Val, 4>;
type ByteHash = Keccak256Hash;
type FieldHash = SerializingHasher<ByteHash>;
type MyCompress = CompressionFunctionFromHasher<ByteHash, 2, 32>;
type Dft = p3_dft::Radix2Bowers;
type ValMmcs = MerkleTreeMmcs<Val, u8, FieldHash, MyCompress, 32>;
type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
type Challenger = SerializingChallenger32<Val, HashChallenger<u8, ByteHash, 32>>;
type Pcs = TwoAdicFriPcs<Val, Dft, ValMmcs, ChallengeMmcs>;
type MyConfig = StarkConfig<Pcs, Challenge, Challenger>;

fn main() -> Result<(), impl Debug> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    let byte_hash = ByteHash {};
    let field_hash = FieldHash::new(Keccak256Hash {});

    let compress = MyCompress::new(byte_hash);

    let val_mmcs = ValMmcs::new(field_hash, compress);

    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());

    let challenger = Challenger::from_hasher(vec![], byte_hash);

    let fri_config = create_benchmark_fri_config(challenge_mmcs);

    let dft = Dft::default();

    let pcs = Pcs::new(dft, val_mmcs, fri_config);

    let config = MyConfig::new(pcs, challenger);

    let starting_value = 52;
    let air = CollatzAir::<N> { starting_value };
    let trace = generate_collatz_trace::<N, Val>(starting_value);

    let proof = prove(&config, &air, trace, &vec![]);

    verify(&config, &air, &proof, &vec![])
}
