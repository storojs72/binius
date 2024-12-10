// Copyright 2024 Irreducible Inc.

use crate::builder::ConstraintSystemBuilder;
use binius_core::oracle::{OracleId, ShiftVariant};
use binius_field::{
	as_packed_field::PackScalar, underlier::UnderlierType, BinaryField1b, TowerField,
};
use binius_macros::arith_expr;
use bytemuck::Pod;
use rayon::prelude::*;

fn u8add_common<U, F>(
	builder: &mut ConstraintSystemBuilder<U, F>,
	xin: OracleId,
	yin: OracleId,
	zout: OracleId,
	cin: OracleId,
	cout: OracleId,
) -> Result<(), anyhow::Error>
where
	U: UnderlierType + Pod + PackScalar<F> + PackScalar<BinaryField1b>,
	F: TowerField,
{
	if let Some(witness) = builder.witness() {
		(
			witness.get::<BinaryField1b>(xin)?.as_slice::<u8>(),
			witness.get::<BinaryField1b>(yin)?.as_slice::<u8>(),
			witness
				.new_column::<BinaryField1b>(zout)
				.as_mut_slice::<u8>(),
			witness
				.new_column::<BinaryField1b>(cout)
				.as_mut_slice::<u8>(),
			witness
				.new_column::<BinaryField1b>(cin)
				.as_mut_slice::<u8>(),
		)
			.into_par_iter()
			.for_each(|(xin, yin, zout, cout, cin)| {
				let carry;
				(*zout, carry) = (*xin).overflowing_add(*yin);
				*cin = (*xin) ^ (*yin) ^ (*zout);
				*cout = ((carry as u8) << 7) | (*cin >> 1);
			});
	}

	builder.assert_zero(
		[xin, yin, cin, cout],
		arith_expr!([xin, yin, cin, cout] = (xin + cin) * (yin + cin) + cin - cout).convert_field(),
	);
	Ok(())
}

pub fn u8add<U, F>(
	builder: &mut ConstraintSystemBuilder<U, F>,
	name: impl ToString,
	xin: OracleId,
	yin: OracleId,
) -> Result<OracleId, anyhow::Error>
where
	U: UnderlierType + Pod + PackScalar<F> + PackScalar<BinaryField1b>,
	F: TowerField,
{
	builder.push_namespace(name);
	let log_rows = builder.log_rows([xin, yin])?;
	let cout = builder.add_committed("cout", log_rows, BinaryField1b::TOWER_LEVEL);
	let cin = builder.add_shifted("cin", cout, 1, 3, ShiftVariant::LogicalLeft)?;

	let zout = builder.add_linear_combination(
		"zout",
		log_rows,
		[(xin, F::ONE), (yin, F::ONE), (cin, F::ONE)].into_iter(),
	)?;

	u8add_common(builder, xin, yin, zout, cin, cout)?;

	builder.pop_namespace();
	Ok(zout)
}

pub fn u8add_committed<U, F>(
	builder: &mut ConstraintSystemBuilder<U, F>,
	name: impl ToString,
	xin: OracleId,
	yin: OracleId,
) -> Result<OracleId, anyhow::Error>
where
	U: UnderlierType + Pod + PackScalar<F> + PackScalar<BinaryField1b>,
	F: TowerField,
{
	builder.push_namespace(name);
	let log_rows = builder.log_rows([xin, yin])?;
	let cout = builder.add_committed("cout", log_rows, BinaryField1b::TOWER_LEVEL);
	let cin = builder.add_shifted("cin", cout, 1, 3, ShiftVariant::LogicalLeft)?;
	let zout = builder.add_committed("zout", log_rows, BinaryField1b::TOWER_LEVEL);

	u8add_common(builder, xin, yin, zout, cin, cout)?;

	builder.assert_zero(
		[xin, yin, cin, zout],
		arith_expr!([xin, yin, cin, zout] = xin + yin + cin - zout).convert_field(),
	);

	builder.pop_namespace();
	Ok(zout)
}
