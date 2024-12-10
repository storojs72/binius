use anyhow::Result;
use binius_circuits::{
	builder::ConstraintSystemBuilder, u8add::u8add_committed, unconstrained::variable_u128,
};
use binius_core::{
	constraint_system, constraint_system::ConstraintSystem, fiat_shamir::HasherChallenger,
	tower::CanonicalTowerFamily, witness::MultilinearExtensionIndex,
};
use binius_field::{arch::OptimalUnderlier, BinaryField128b, BinaryField1b, BinaryField8b};
use binius_hal::make_portable_backend;
use binius_hash::{GroestlDigestCompression, GroestlHasher};
use binius_math::DefaultEvaluationDomainFactory;
use groestl_crypto::Groestl256;

const LOG_SIZE: usize = 10;

fn out_of_circuit(x: u8, y: u8) -> u8 {
	x + y
}

fn cs_builder(x: u8, y: u8) -> u8 {
	let allocator = bumpalo::Bump::new();
	let mut builder =
		ConstraintSystemBuilder::<OptimalUnderlier, BinaryField128b>::new_with_witness(&allocator);

	let x = variable_u128::<_, _, BinaryField1b>(&mut builder, "x", LOG_SIZE, x as u128).unwrap();
	let y = variable_u128::<_, _, BinaryField1b>(&mut builder, "y", LOG_SIZE, y as u128).unwrap();

	let sum = u8add_committed(&mut builder, "x + y", x, y).unwrap();

	let witness = builder.witness().unwrap();

	let sum_value = witness
		.get::<BinaryField1b>(sum)
		.unwrap()
		.as_slice::<u128>();
	assert!(sum_value.len() > 0);

	let witness = builder.take_witness().unwrap();
	let cs = builder.build().unwrap();

	prove_verify_test(cs, witness).unwrap();

	sum_value[0] as u8
}

fn prove_verify_test(
	cs: ConstraintSystem<BinaryField128b>,
	witness: MultilinearExtensionIndex<OptimalUnderlier, BinaryField128b>,
) -> Result<()> {
	let backend = make_portable_backend();
	let domain_factory = DefaultEvaluationDomainFactory::default();

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
	>(&cs, 1usize, 100usize, witness, &domain_factory, &backend)?;

	constraint_system::verify::<
		OptimalUnderlier,
		CanonicalTowerFamily,
		_,
		_,
		GroestlHasher<BinaryField128b>,
		GroestlDigestCompression<BinaryField8b>,
		HasherChallenger<Groestl256>,
	>(&cs, 1usize, 100usize, &domain_factory, vec![], proof)?;

	println!("success");

	Ok(())
}

fn main() {
	let x = 100u8;
	let y = 200u8;
	let out = out_of_circuit(x, y);

	let out_cs_builder = cs_builder(x, y);
	assert_eq!(out, out_cs_builder);
}
