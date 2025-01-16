// Copyright 2024-2025 Irreducible Inc.
use binius_core::oracle::OracleId;
use binius_field::{
	as_packed_field::PackScalar, underlier::UnderlierType, ExtensionField, TowerField,
};
use bytemuck::Pod;
use rand::{thread_rng, Rng};
use rayon::prelude::*;

use crate::builder::ConstraintSystemBuilder;

pub fn unconstrained<U, F, FS>(
	builder: &mut ConstraintSystemBuilder<U, F>,
	name: impl ToString,
	log_size: usize,
) -> Result<OracleId, anyhow::Error>
where
	U: UnderlierType + Pod + PackScalar<F> + PackScalar<FS>,
	F: TowerField + ExtensionField<FS>,
	FS: TowerField,
{
	let rng = builder.add_committed(name, log_size, FS::TOWER_LEVEL);

	if let Some(witness) = builder.witness() {
		witness
			.new_column::<FS>(rng)
			.as_mut_slice::<u8>()
			.into_par_iter()
			.for_each_init(thread_rng, |rng, data| {
				*data = rng.gen();
			});
	}

	Ok(rng)
}

pub fn variable<U, F, FS>(
	builder: &mut ConstraintSystemBuilder<U, F>,
	name: impl ToString,
	log_size: usize,
	value: u32,
) -> Result<OracleId, anyhow::Error>
where
	U: UnderlierType + Pod + PackScalar<F> + PackScalar<FS>,
	F: TowerField + ExtensionField<FS>,
	FS: TowerField,
{
	variable_u32::<U, F, FS>(builder, name, log_size, value)
}

pub fn variable_u128<U, F, FS>(
	builder: &mut ConstraintSystemBuilder<U, F>,
	name: impl ToString,
	log_size: usize,
	value: u128,
) -> Result<OracleId, anyhow::Error>
where
	U: UnderlierType + Pod + PackScalar<F> + PackScalar<FS>,
	F: TowerField + ExtensionField<FS>,
	FS: TowerField,
{
	let rng = builder.add_committed(name, log_size, FS::TOWER_LEVEL);

	if let Some(witness) = builder.witness() {
		witness
			.new_column::<FS>(rng)
			.as_mut_slice::<u128>()
			.into_par_iter()
			.for_each_init(thread_rng, |_rng, data| {
				*data = value;
			});
	}

	Ok(rng)
}

pub fn variable_u64<U, F, FS>(
	builder: &mut ConstraintSystemBuilder<U, F>,
	name: impl ToString,
	log_size: usize,
	value: u64,
) -> Result<OracleId, anyhow::Error>
where
	U: UnderlierType + Pod + PackScalar<F> + PackScalar<FS>,
	F: TowerField + ExtensionField<FS>,
	FS: TowerField,
{
	let rng = builder.add_committed(name, log_size, FS::TOWER_LEVEL);

	if let Some(witness) = builder.witness() {
		witness
			.new_column::<FS>(rng)
			.as_mut_slice::<u64>()
			.into_par_iter()
			.for_each_init(thread_rng, |_rng, data| {
				*data = value;
			});
	}

	Ok(rng)
}

pub fn variable_u32<U, F, FS>(
	builder: &mut ConstraintSystemBuilder<U, F>,
	name: impl ToString,
	log_size: usize,
	value: u32,
) -> Result<OracleId, anyhow::Error>
where
	U: UnderlierType + Pod + PackScalar<F> + PackScalar<FS>,
	F: TowerField + ExtensionField<FS>,
	FS: TowerField,
{
	let rng = builder.add_committed(name, log_size, FS::TOWER_LEVEL);

	if let Some(witness) = builder.witness() {
		witness
			.new_column::<FS>(rng)
			.as_mut_slice::<u32>()
			.into_par_iter()
			.for_each_init(thread_rng, |_rng, data| {
				*data = value;
			});
	}

	Ok(rng)
}
