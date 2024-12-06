use binius_circuits::{
	builder::ConstraintSystemBuilder,
	u32add::u32add_committed,
	unconstrained::{variable, variable_u128, variable_u32},
};
use binius_field::{
	arch::{OptimalUnderlier, OptimalUnderlier256b},
	BinaryField128b, BinaryField1b, BinaryField64b, BinaryField8b,
};

// this constant defines how much is the memory allocated for the variables in constraints system, for example for OptimalUnderlier (OptimalUnderlier128b):
// 5 -> [0u128; 1]
// 8 -> [0u128; 2]
// 10 -> [0u128; 8]
// 11 -> [0u128; 16],
const LOG_SIZE: usize = 5;

fn out_of_circuit_computation() -> u32 {
	let x = 25u32;
	let y = 50u32;
	x + y
}

// OptimalUnderlier defines actual "size" of single variable memory in the constraint system.
// it can be 128b, 256b or 512b.
fn in_circuit_computation() -> u32 {
	let x = 25u128;
	let y = 50u128;

	let allocator = bumpalo::Bump::new();
	let mut builder =
		ConstraintSystemBuilder::<OptimalUnderlier, BinaryField128b>::new_with_witness(&allocator);

	let x = variable_u128::<_, _, BinaryField1b>(&mut builder, "x", LOG_SIZE, x).unwrap();
	let y = variable_u128::<_, _, BinaryField1b>(&mut builder, "y", LOG_SIZE, y).unwrap();
	let sum = u32add_committed(&mut builder, "x + y", x, y).unwrap();
	//let sum = u32add_committed(&mut builder, "sum + x", sum, x).unwrap();

	let witness = builder.witness().unwrap();
	let sum_value = witness
		.get::<BinaryField1b>(sum)
		.unwrap()
		.as_slice::<u128>();
	assert!(sum_value.len() > 0);

	let witness = builder.take_witness().unwrap();
	println!("witness: {:?}", witness);
	let cs = builder.build().unwrap();
	println!("constraints system: {:?}", cs);

	sum_value[0] as u32
}

fn main() {
	let a = out_of_circuit_computation();
	let b = in_circuit_computation();
	assert_eq!(a, b);
	println!("success");
}
