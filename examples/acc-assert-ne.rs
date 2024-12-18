use binius_circuits::{builder::ConstraintSystemBuilder, unconstrained::variable_u128};
use binius_core::{
	constraint_system, constraint_system::ConstraintSystem, fiat_shamir::HasherChallenger,
	oracle::OracleId, tower::CanonicalTowerFamily, witness::MultilinearExtensionIndex,
};
use binius_field::{arch::OptimalUnderlier, BinaryField128b, BinaryField1b, BinaryField8b};
use binius_hal::make_portable_backend;
use binius_hash::{GroestlDigestCompression, GroestlHasher};
use binius_math::{ArithExpr::Var, DefaultEvaluationDomainFactory};
use groestl_crypto::Groestl256;

const LOG_SIZE: usize = 10;

const MASK: [u128; 128] = [
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b00100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b01000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    0b10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
];

fn assert_ne_gadget(
	builder: &mut ConstraintSystemBuilder<OptimalUnderlier, BinaryField128b>,
	input: &[bool],
) {
	fn set_bit(a: u128, index: usize) -> u128 {
		assert!(index < 128);
		let out = a | MASK[index];
		out
	}

	assert!(input.len() > 0);

	let ids = input
		.chunks(128)
		.into_iter()
		.enumerate()
		.map(|(chunk_index, chunk)| {
			let mut value = 0u128;
			for (bit_index, one) in chunk.iter().enumerate() {
				if *one {
					value = set_bit(value, bit_index);
				}
			}
			let id = if value != 0u128 {
				variable_u128::<_, _, BinaryField1b>(
					builder,
					format!("packed_value::{}", chunk_index),
					LOG_SIZE,
					1u128,
				)
				.unwrap()
			} else {
				variable_u128::<_, _, BinaryField1b>(
					builder,
					format!("packed_value::{}", chunk_index),
					LOG_SIZE,
					0u128,
				)
				.unwrap()
			};
			id
		})
		.collect::<Vec<OracleId>>();

	// Constraint:
	//
	// NOT (x_0 OR x_1 OR x_2 OR ... OR x_n) == 0, where
	//
	// NOT x = x XOR 1 (considering X as a bit),
	// x_0 OR x_1 = (x_0 XOR x_1) XOR (x_0 AND x_1).

	let mut composition = Var(ids[0].clone());
	for id in ids.iter() {
		composition = composition.clone() + Var(*id) + composition.clone() * Var(*id);
	}

	let one = variable_u128::<_, _, BinaryField1b>(builder, "one", LOG_SIZE, 1u128).unwrap();

	// FIXME: Why 'composition - Const(1)' not possible?
	builder.assert_zero_acc([ids, vec![one]].concat(), composition - Var(one));
}

fn main() {
	// Positive test (at least one non-zero bit in the input)
	let mut input = [false; 128];
	input[0] = true;

	let allocator = bumpalo::Bump::new();
	let mut builder =
		ConstraintSystemBuilder::<OptimalUnderlier, BinaryField128b>::new_with_witness(&allocator);

	assert_ne_gadget(&mut builder, &input);

	let witness = builder.take_witness().unwrap();
	let cs = builder.build().unwrap();

	let (prove_no_issues, verify_no_issues) = prove_verify_test(witness, cs);
	assert!(prove_no_issues);
	assert!(verify_no_issues);
	println!("ok");

	// Positive test (at least one non-zero bit in the input)
	let mut input = [false; 400];
	input[input.len() - 1] = true;

	let allocator = bumpalo::Bump::new();
	let mut builder =
		ConstraintSystemBuilder::<OptimalUnderlier, BinaryField128b>::new_with_witness(&allocator);

	assert_ne_gadget(&mut builder, &input);

	let witness = builder.take_witness().unwrap();
	let cs = builder.build().unwrap();

	let (prove_no_issues, verify_no_issues) = prove_verify_test(witness, cs);
	assert!(prove_no_issues);
	assert!(verify_no_issues);
	println!("ok");

	// Negative test (verification fails if all zeroes in the input)
	let input = [false; 512];

	let allocator = bumpalo::Bump::new();
	let mut builder =
		ConstraintSystemBuilder::<OptimalUnderlier, BinaryField128b>::new_with_witness(&allocator);

	assert_ne_gadget(&mut builder, &input);

	let witness = builder.take_witness().unwrap();
	let cs = builder.build().unwrap();

	let (prove_no_issues, verify_no_issues) = prove_verify_test(witness, cs);
	assert!(prove_no_issues);
	assert!(!verify_no_issues);
	println!("ok");

	// Negative test (verification fails if all zeroes in the input)
	let input = [false; 1000];

	let allocator = bumpalo::Bump::new();
	let mut builder =
		ConstraintSystemBuilder::<OptimalUnderlier, BinaryField128b>::new_with_witness(&allocator);

	assert_ne_gadget(&mut builder, &input);

	let witness = builder.take_witness().unwrap();
	let cs = builder.build().unwrap();

	let (prove_no_issues, verify_no_issues) = prove_verify_test(witness, cs);
	assert!(prove_no_issues);
	assert!(!verify_no_issues);
	println!("ok");

	// FIXME: segfault occurs when using inputs with 2000 and more bits
}

fn prove_verify_test(
	witness: MultilinearExtensionIndex<OptimalUnderlier, BinaryField128b>,
	constraints: ConstraintSystem<BinaryField128b>,
) -> (bool, bool) {
	let domain_factory = DefaultEvaluationDomainFactory::default();
	let backend = make_portable_backend();
	let proof = constraint_system::prove::<
		OptimalUnderlier,
		CanonicalTowerFamily,
		BinaryField8b,
		_,
		_,
		GroestlHasher<BinaryField128b>,
		GroestlDigestCompression<BinaryField8b>,
		HasherChallenger<Groestl256>,
		_,
	>(&constraints, 1usize, 100usize, witness, &domain_factory, &backend);

	let prove_no_issues = proof.is_ok();
	if !prove_no_issues {
		println!("{:?}", proof);
		// Since we have issue on proving, verification is also an issue
		return (false, false);
	}

	let out = constraint_system::verify::<
		OptimalUnderlier,
		CanonicalTowerFamily,
		_,
		_,
		GroestlHasher<BinaryField128b>,
		GroestlDigestCompression<BinaryField8b>,
		HasherChallenger<Groestl256>,
	>(&constraints, 1usize, 100usize, &domain_factory, vec![], proof.unwrap());

	let verify_no_issues = out.is_ok();
	(prove_no_issues, verify_no_issues)
}
