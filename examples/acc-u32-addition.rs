use anyhow::Result;
use binius_circuits::{arithmetic, builder::ConstraintSystemBuilder, unconstrained::variable_u128};
use binius_core::{
	constraint_system,
	constraint_system::ConstraintSystem,
	fiat_shamir::HasherChallenger,
	tower::CanonicalTowerFamily,
	witness::MultilinearExtensionIndex,
};
use binius_field::{
	arch::OptimalUnderlier, BinaryField128b, BinaryField1b, BinaryField8b,
};
use binius_hal::make_portable_backend;
use binius_hash::{GroestlDigestCompression, GroestlHasher};
use binius_math::DefaultEvaluationDomainFactory;
use groestl_crypto::Groestl256;
use binius_acc_utils::prove_verify_test;

const ROWS: usize = 7;

fn out_of_circuit(x: u32, y: u32) -> u32 {
	x + y
}

fn u32_addition(x: u32, y: u32) -> u32 {
	let allocator = bumpalo::Bump::new();
	let mut builder =
		ConstraintSystemBuilder::<OptimalUnderlier, BinaryField128b>::new_with_witness(&allocator);

	let x = variable_u128::<_, _, BinaryField1b>(&mut builder, "x", ROWS, x as u128).unwrap();
	let y = variable_u128::<_, _, BinaryField1b>(&mut builder, "y", ROWS, y as u128).unwrap();

	let sum = arithmetic::u32::add(&mut builder, "x + y", x, y, arithmetic::Flags::Unchecked).unwrap();

	let witness = builder.witness().unwrap();
	let sum_value = witness
		.get::<BinaryField1b>(sum)
		.unwrap()
		.as_slice::<u128>();
	assert!(sum_value.len() > 0);

	let witness = builder.take_witness().unwrap();
	//println!("witness: {:?}", witness);

	let cs = builder.build().unwrap();

	//print!("cs_builder: ");
	prove_verify_test(cs, witness);

	sum_value[0] as u32
}

fn u32_addition_lookup(x: u32, y: u32) -> u32 {
	x + y
}

fn main() {
	let x = 100u32;
	let y = 200u32;

	let out1 = out_of_circuit(x, y);
	let out2 = u32_addition(x, y);
	assert_eq!(out1, out2);


	let out3 = u32_addition_lookup(x, y);
	assert_eq!(out1, out3);
}
