use binius_acc_utils::prove_verify_test;
use binius_circuits::{builder::ConstraintSystemBuilder, unconstrained::variable_u128};
use binius_field::{arch::OptimalUnderlier, BinaryField128b, BinaryField1b};
use binius_macros::arith_expr;

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

fn assert_eq_gadget(
	builder: &mut ConstraintSystemBuilder<OptimalUnderlier, BinaryField128b>,
	input: &[bool],
) {
	assert!(input.len() > 0);

	fn set_bit(a: u128, index: usize) -> u128 {
		assert!(index < 128);
		let out = a | MASK[index];
		out
	}

	// Split input into 128-bit chunks, instantiate variable by setting its bits from the input and finally constraint it to be zero
	input
		.chunks(128)
		.into_iter()
		.enumerate()
		.for_each(|(chunk_index, chunk)| {
			let mut value = 0u128;
			for (bit_index, one) in chunk.iter().enumerate() {
				if *one {
					value = set_bit(value, bit_index);
				}
			}
			let value_id = variable_u128::<_, _, BinaryField1b>(
				builder,
				format!("packed_value::{}", chunk_index),
				LOG_SIZE,
				value,
			)
			.unwrap();
			builder.assert_zero([value_id], arith_expr!([val] = val - 0).convert_field());
		});
}

fn main() {
	fn positive_test(input: &[bool]) {
		let allocator = bumpalo::Bump::new();
		let mut builder =
			ConstraintSystemBuilder::<OptimalUnderlier, BinaryField128b>::new_with_witness(
				&allocator,
			);

		assert_eq_gadget(&mut builder, input);

		let witness = builder.take_witness().unwrap();
		let cs = builder.build().unwrap();

		let (prove_no_issues, verify_no_issues) = prove_verify_test(witness, cs);
		assert!(prove_no_issues);
		assert!(verify_no_issues);
	}

	fn negative_test(input: &[bool]) {
		let allocator = bumpalo::Bump::new();
		let mut builder =
			ConstraintSystemBuilder::<OptimalUnderlier, BinaryField128b>::new_with_witness(
				&allocator,
			);

		assert_eq_gadget(&mut builder, input);

		let witness = builder.take_witness().unwrap();
		let cs = builder.build().unwrap();

		let (prove_no_issues, verify_no_issues) = prove_verify_test(witness, cs);
		assert!(prove_no_issues);
		assert!(!verify_no_issues); // issue on verification is expected
	}

	println!("Start");

	let input = [false; 1];
	positive_test(&input);
	println!("OK");

	let input = [false; 2];
	positive_test(&input);
	println!("OK");

	let input = [false; 10];
	positive_test(&input);
	println!("OK");

	let input = [false; 256];
	positive_test(&input);
	println!("OK");

	let input = [false; 100000];
	positive_test(&input);
	println!("OK");

	let mut input = [false; 100000];
	input[9999] = true;
	negative_test(&input);
	println!("OK");

	let input = [true; 1];
	negative_test(&input);
	println!("OK");

	let input = [true; 2];
	negative_test(&input);
	println!("OK");

	let input = [true; 10];
	negative_test(&input);
	println!("OK");

	let input = [true; 100000];
	negative_test(&input);
	println!("OK");

	let mut input = [false; 10];
	input[0] = true;
	input[2] = true;
	negative_test(&input);
	println!("OK");

	input[3] = true;
	negative_test(&input);
	println!("OK");
	println!("Done");
}
