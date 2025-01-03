use binius_circuits::{
	builder::ConstraintSystemBuilder,
	unconstrained::variable_u128,
};
use binius_core::{
	constraint_system, constraint_system::ConstraintSystem, fiat_shamir::HasherChallenger,
	oracle::OracleId, tower::CanonicalTowerFamily, witness::MultilinearExtensionIndex,
};
use binius_field::{
	arch::OptimalUnderlier, BinaryField128b, BinaryField1b, BinaryField8b,
};
use binius_hal::make_portable_backend;
use binius_macros::arith_expr;
use groestl_crypto::Groestl256;
use std::time::Instant;
use binius_hash::compress::Groestl256ByteCompression;
use binius_math::DefaultEvaluationDomainFactory;

const LOG_SIZE: usize = 10;

fn prove_verify_test(
	witness: MultilinearExtensionIndex<OptimalUnderlier, BinaryField128b>,
	constraints: ConstraintSystem<BinaryField128b>,
) -> (bool, bool) {
	let domain_factory = DefaultEvaluationDomainFactory::default();
	let backend = make_portable_backend();

	let start = Instant::now();
	let proof = constraint_system::prove::<
		OptimalUnderlier,
		CanonicalTowerFamily,
		BinaryField8b,
		_,
		Groestl256,
		Groestl256ByteCompression,
		HasherChallenger<Groestl256>,
		_,
	>(&constraints, 1usize, 100usize, witness, &domain_factory, &backend);
	println!("proving done in {} ms", start.elapsed().as_millis());

	let prove_no_issues = proof.is_ok();
	if !prove_no_issues {
		// Since we have issue on proving, verification is also an issue
		return (false, false);
	}

	let start = Instant::now();
	let out = constraint_system::verify::<
		OptimalUnderlier,
		CanonicalTowerFamily,
		Groestl256,
		Groestl256ByteCompression,
		HasherChallenger<Groestl256>,
	>(&constraints, 1usize, 100usize, vec![], proof.unwrap());
	println!("verification done in {} ms", start.elapsed().as_millis());
	println!();

	let verify_no_issues = out.is_ok();
	(prove_no_issues, verify_no_issues)
}

fn benchmark_constraints(n: usize) {
	let allocator = bumpalo::Bump::new();
	let mut builder =
		ConstraintSystemBuilder::<OptimalUnderlier, BinaryField128b>::new_with_witness(&allocator);

	let x = variable_u128::<_, _, BinaryField1b>(&mut builder, "x", LOG_SIZE, 100u128).unwrap();
	let y = variable_u128::<_, _, BinaryField1b>(&mut builder, "y", LOG_SIZE, 200u128).unwrap();
	let composition = arith_expr!([x, y] = x + x - y - y + 0);

	for _ in 0..n {
		builder.assert_zero([x, y], composition.clone().convert_field());
	}

	let witness = builder.take_witness().unwrap();
	let cs = builder.build().unwrap();

	let (prove_ok, verify_ok) = prove_verify_test(witness, cs);
	assert!(prove_ok);
	assert!(verify_ok);
}

fn benchmark_variables<const N: usize>(input: [u128; N]) {
	assert!(N > 2);
	let allocator = bumpalo::Bump::new();
	let mut builder =
		ConstraintSystemBuilder::<OptimalUnderlier, BinaryField128b>::new_with_witness(&allocator);

	let vars: [OracleId; N] = input
		.into_iter()
		.enumerate()
		.map(|(index, item)| {
			variable_u128::<_, _, BinaryField1b>(
				&mut builder,
				format!("var {}", index),
				LOG_SIZE,
				item,
			)
				.unwrap()
		})
		.collect::<Vec<OracleId>>()
		.try_into()
		.unwrap();

	//let x = vars[0];
	//let y = vars[1];
	let composition = arith_expr!([x, y] = x + x - y - y);

	builder.assert_zero(vars, composition.convert_field());

	let witness = builder.take_witness().unwrap();
	let cs = builder.build().unwrap();

	let (prove_ok, verify_ok) = prove_verify_test(witness, cs);
	assert!(prove_ok);
	assert!(verify_ok);
}

fn main() {
	println!("1000 variables");
	let input = [1u128; 1000];
	benchmark_variables(input);

	println!("10000 variables");
	let input = [1u128; 10000];
	benchmark_variables(input);

	println!("50000 variables");
	let input = [1u128; 50000];
	benchmark_variables(input);

    // TODO: segfault begins happening
	println!("100000 variables");
	let input = [1u128; 100000];
	benchmark_variables(input);

	println!("1000 constraints");
	benchmark_constraints(1000);

	println!("10000 constraints");
	benchmark_constraints(10000);

	println!("50000 constraints");
	benchmark_constraints(50000);

	println!("100000 constraints");
	benchmark_constraints(100000);
}
