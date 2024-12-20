// Copyright 2024 Irreducible Inc.

use std::array;

use anyhow::Result;
use binius_circuits::{builder::ConstraintSystemBuilder, unconstrained::unconstrained};
use binius_core::{constraint_system, fiat_shamir::HasherChallenger, tower::CanonicalTowerFamily};
use binius_field::{arch::OptimalUnderlier, BinaryField128b, BinaryField1b, BinaryField8b};
use binius_hal::make_portable_backend;
use binius_hash::{GroestlDigestCompression, GroestlHasher};
use binius_math::DefaultEvaluationDomainFactory;
use binius_utils::{checked_arithmetics::log2_ceil_usize, rayon::adjust_thread_pool};
use clap::{value_parser, Parser};
use groestl_crypto::Groestl256;
use tracing_profile::init_tracing;

const LOG_ROWS_PER_PERMUTATION: usize = 6;

#[derive(Debug, Parser)]
struct Args {
	/// The number of permutations to verify.
	#[arg(short, long, default_value_t = 128, value_parser = value_parser!(u32).range(1 << 3..))]
	n_permutations: u32,
	/// The negative binary logarithm of the Reed–Solomon code rate.
	#[arg(long, default_value_t = 1, value_parser = value_parser!(u32).range(1..))]
	log_inv_rate: u32,
}

fn main() -> Result<()> {
	type U = OptimalUnderlier;
	const SECURITY_BITS: usize = 100;

	adjust_thread_pool()
		.as_ref()
		.expect("failed to init thread pool");

	let args = Args::parse();

	let _guard = init_tracing().expect("failed to initialize tracing");

	println!("Verifying {} Keccak-f permutations", args.n_permutations);

	let log_n_permutations = log2_ceil_usize(args.n_permutations as usize);

	let allocator = bumpalo::Bump::new();

	let mut builder = ConstraintSystemBuilder::<U, BinaryField128b>::new_with_witness(&allocator);

	let log_size = log_n_permutations + LOG_ROWS_PER_PERMUTATION;

	let input = array::from_fn(|_| {
		unconstrained::<_, _, BinaryField1b>(&mut builder, "input", log_size).unwrap()
	});

	let _state_out = binius_circuits::keccakf::keccakf(&mut builder, input, log_size);

	let witness = builder
		.take_witness()
		.expect("builder created with witness");
	let constraint_system = builder.build()?;

	let domain_factory = DefaultEvaluationDomainFactory::default();
	let backend = make_portable_backend();

	let proof = constraint_system::prove::<
		U,
		CanonicalTowerFamily,
		BinaryField8b,
		_,
		_,
		GroestlHasher<BinaryField128b>,
		GroestlDigestCompression<BinaryField8b>,
		HasherChallenger<Groestl256>,
		_,
	>(
		&constraint_system,
		args.log_inv_rate as usize,
		SECURITY_BITS,
		witness,
		&domain_factory,
		&backend,
	)?;

	constraint_system::verify::<
		U,
		CanonicalTowerFamily,
		_,
		_,
		GroestlHasher<BinaryField128b>,
		GroestlDigestCompression<BinaryField8b>,
		HasherChallenger<Groestl256>,
	>(
		&constraint_system.no_base_constraints(),
		args.log_inv_rate as usize,
		SECURITY_BITS,
		&domain_factory,
		vec![],
		proof,
	)?;

	Ok(())
}
