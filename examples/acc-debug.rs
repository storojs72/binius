use binius_acc_utils::prove_verify_test;
use binius_circuits::builder::ConstraintSystemBuilder;
use binius_core::constraint_system::validate::validate_witness;
use binius_field::{
	arch::OptimalUnderlier, as_packed_field::PackScalar, underlier::UnderlierType, BinaryField128b,
	BinaryField1b, ExtensionField, TowerField,
};
use binius_macros::arith_expr;
use bytemuck::Pod;

fn gadget<U, F, FS>(builder: &mut ConstraintSystemBuilder<U, F>, n_vars: usize)
where
	U: UnderlierType + Pod + PackScalar<F> + PackScalar<FS>,
	F: TowerField + ExtensionField<FS>,
	FS: TowerField,
{
	// define variable
	let variable = builder.add_committed("variable", n_vars, FS::TOWER_LEVEL);

	// write some data to the variable
	if let Some(witness) = builder.witness() {
		witness
			.new_column::<FS>(variable)
			.as_mut_slice::<u8>()
			.into_iter()
			.for_each(|entry| *entry = 1u8)
	};

	// set constraint for the variable
	builder.assert_zero(
		"simple constraint",
		vec![variable],
		arith_expr!([variable] = variable - variable).convert_field(),
	);
}

fn main() {
	type CsF = BinaryField128b;
	type VarF = BinaryField1b;

	// Expected number of rows in the constraint system
	let n_vars = 10usize;

	let allocator = bumpalo::Bump::new();
	let mut builder =
		ConstraintSystemBuilder::<OptimalUnderlier, CsF>::new_with_witness(&allocator);

	gadget::<OptimalUnderlier, CsF, VarF>(&mut builder, n_vars);

	let witness = builder.take_witness().unwrap();
	let constraints_system = builder.build().unwrap();

	validate_witness(&constraints_system, &[], &witness).unwrap();

	let (prove_is_ok, verify_is_ok) = prove_verify_test(witness, constraints_system);
	assert!(prove_is_ok);
	assert!(verify_is_ok);
}
