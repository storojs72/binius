use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use binius_circuits::builder::{witness::Builder, ConstraintSystemBuilder};
use binius_core::{
	constraint_system::{
		channel::{Boundary, ChannelId, FlushDirection},
		validate::validate_witness,
	},
	oracle::{MultilinearOracleSet, OracleId, ShiftVariant},
	witness::MultilinearExtensionIndex,
};
use binius_field::{
	arch::OptimalUnderlier, as_packed_field::PackScalar, BinaryField128b, BinaryField32b,
	ExtensionField, TowerField,
};
use binius_utils::checked_arithmetics::log2_ceil_usize;
use bumpalo::Bump;
use bytemuck::Pod;

struct Oracles {
	n: OracleId,
	s: OracleId,
	n_next: OracleId,
	s_next: OracleId,
}

fn constrain<U, F>(
	builder: &mut ConstraintSystemBuilder<U, F>,
	count: usize,
) -> Result<(Oracles, ChannelId)>
where
	U: PackScalar<F>,
	F: TowerField,
{
	let log_size = log2_ceil_usize(count);
	let n = builder.add_committed("n", log_size, BinaryField32b::TOWER_LEVEL);
	let s = builder.add_committed("s", log_size, BinaryField32b::TOWER_LEVEL);
	let n_next = builder.add_shifted("n_next", n, 1, log_size, ShiftVariant::LogicalRight)?;
	let s_next = builder.add_shifted("s_next", s, 1, log_size, ShiftVariant::LogicalRight)?;

	builder.assert_not_zero(n);

	let channel = builder.add_channel();
	builder.send(channel, count, [n, s]);
	builder.receive(channel, count, [n_next, s_next]);
	Ok((
		Oracles {
			n,
			s,
			n_next,
			s_next,
		},
		channel,
	))
}

fn synthesize<
	'a,
	F: TowerField + ExtensionField<BinaryField32b>,
	U: PackScalar<F> + PackScalar<BinaryField32b> + Pod,
>(
	allocator: &'a Bump,
	oracles_set: MultilinearOracleSet<F>,
	oracles: Oracles,
	ns: &[u32],
	ss: &[u32],
) -> Result<MultilinearExtensionIndex<'a, U, F>> {
	assert_eq!(ns.len(), ss.len());
	let witness = Builder::new(allocator, Rc::new(RefCell::new(oracles_set)));
	let Oracles {
		n,
		s,
		n_next,
		s_next,
	} = oracles;

	witness
		.new_column::<BinaryField32b>(n)
		.as_mut_slice()
		.copy_from_slice(&ns[..ns.len() - 1]);

	witness
		.new_column::<BinaryField32b>(s)
		.as_mut_slice()
		.copy_from_slice(&ss[..ss.len() - 1]);

	witness
		.new_column::<BinaryField32b>(n_next)
		.as_mut_slice()
		.copy_from_slice(&ns[1..]);

	witness
		.new_column::<BinaryField32b>(s_next)
		.as_mut_slice()
		.copy_from_slice(&ss[1..]);

	witness.build()
}

fn main() {
	let n = 8u32;
	let mut builder = ConstraintSystemBuilder::<OptimalUnderlier, BinaryField128b>::new();
	let (oracles, channel_id) = constrain(&mut builder, n as usize).unwrap();

	let cs = builder.build().unwrap();

	// [8, 7, 6, 5, 4, 3, 2, 1, 0]
	let ns: Vec<u32> = (0..=n).rev().collect();

	// [36, 28, 21, 15, 10, 6, 3, 1, 0]
	let ss: Vec<u32> = ns.iter().map(|n| n * (n + 1) / 2).collect();
	let allocator = Bump::new();
	let witness: MultilinearExtensionIndex<'_, OptimalUnderlier, _> =
		synthesize(&allocator, cs.oracles.clone(), oracles, &ns, &ss).unwrap();

	let f = |x| BinaryField32b::new(x).into();

	let boundaries = [
		Boundary {
			values: vec![f(n), f(ss[0])],
			channel_id,
			direction: FlushDirection::Pull,
			multiplicity: 1,
		},
		Boundary {
			values: vec![f(0), f(0)],
			channel_id,
			direction: FlushDirection::Push,
			multiplicity: 1,
		},
	];

	validate_witness(&cs, &boundaries, &witness).unwrap();
}
