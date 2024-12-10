use anyhow::Result;
use binius_circuits::{
	builder::{witness::WitnessEntry, ConstraintSystemBuilder},
	u32add::u32add_committed,
	unconstrained::{variable, variable_u128},
};
use binius_core::{
	constraint_system,
	constraint_system::ConstraintSystem,
	fiat_shamir::HasherChallenger,
	oracle::{
		CommittedBatch, CommittedBatchMeta, CommittedId, Constraint, ConstraintPredicate,
		ConstraintSet, MultilinearOracleMeta, MultilinearOracleSet, MultilinearPolyOracle,
		MultilinearPolyOracle::Committed, OracleId, ShiftVariant,
	},
	tower::CanonicalTowerFamily,
	witness::{MultilinearExtensionIndex, MultilinearWitness},
};
use binius_field::{
	arch::OptimalUnderlier, BinaryField128b, BinaryField1b, BinaryField8b, PackedBinaryField128x1b,
	PackedBinaryField16x8b, PackedBinaryField1x128b,
};
use binius_hal::make_portable_backend;
use binius_hash::{GroestlDigestCompression, GroestlHasher};
use binius_math::{
	ArithExpr::{Add, Mul, Var},
	DefaultEvaluationDomainFactory, MLEEmbeddingAdapter, MultilinearExtension,
	MultilinearExtensionBorrowed,
};
use groestl_crypto::Groestl256;
use rayon::yield_local;

const LOG_SIZE: usize = 10;

fn out_of_circuit(x: u32, y: u32) -> u32 {
	x + y
}

fn cs_builder(x: u32, y: u32) -> u32 {
	let allocator = bumpalo::Bump::new();
	let mut builder =
		ConstraintSystemBuilder::<OptimalUnderlier, BinaryField128b>::new_with_witness(&allocator);

	let x = variable_u128::<_, _, BinaryField1b>(&mut builder, "x", LOG_SIZE, x as u128).unwrap();
	let y = variable_u128::<_, _, BinaryField1b>(&mut builder, "y", LOG_SIZE, y as u128).unwrap();

	let sum = u32add_committed(&mut builder, "x + y", x, y).unwrap();

	let witness = builder.witness().unwrap();
	let sum_value = witness
		.get::<BinaryField1b>(sum)
		.unwrap()
		.as_slice::<u128>();
	assert!(sum_value.len() > 0);

	let witness = builder.take_witness().unwrap();
	let cs = builder.build().unwrap();

	print!("cs_builder: ");
	prove_verify_test(cs, witness).unwrap();

	sum_value[0] as u32
}

fn manual_cs(x: u32, y: u32) -> u32 {
	//25 + 50 = 75
	//
	//25, 50, 48, 96, 75

	// instantiate witness
	let (in4, carry) = x.overflowing_add(y);
	let in3 = x ^ y ^ in4;
	let in2 = ((carry as u32) << 31) | in3 >> 1;
	let in1 = y;
	let in0 = x;

	let mut witness = MultilinearExtensionIndex::<OptimalUnderlier, BinaryField128b>::new();
	for item in [in0, in1, in2, in3, in4] {
		let ext = MultilinearExtension {
			mu: 10usize,
			evals: vec![
				PackedBinaryField128x1b::from(item as u128),
				PackedBinaryField128x1b::from(item as u128),
				PackedBinaryField128x1b::from(item as u128),
				PackedBinaryField128x1b::from(item as u128),
				PackedBinaryField128x1b::from(item as u128),
				PackedBinaryField128x1b::from(item as u128),
				PackedBinaryField128x1b::from(item as u128),
				PackedBinaryField128x1b::from(item as u128),
			],
		};
		let ext = MLEEmbeddingAdapter::<_, PackedBinaryField1x128b, _>::from(ext);
		let mlw: MultilinearWitness<PackedBinaryField1x128b> = ext.upcast_arc_dyn();
		witness.entries.push(Some(mlw));
	}

	// instantiate constraints system
	let mut oracle_set = MultilinearOracleSet::new();
	oracle_set.batches.push(CommittedBatchMeta {
		oracle_ids: vec![
			OracleId::from(0usize),
			OracleId::from(1usize),
			OracleId::from(2usize),
			OracleId::from(4usize),
		],
		n_vars: LOG_SIZE,
		tower_level: 0,
	});
	oracle_set.oracles.push(MultilinearOracleMeta::Committed {
		committed_id: CommittedId {
			batch_id: 0,
			index: 0,
		},
		name: Some("x".to_string()),
	});
	oracle_set.oracles.push(MultilinearOracleMeta::Committed {
		committed_id: CommittedId {
			batch_id: 0,
			index: 1,
		},
		name: Some("y".to_string()),
	});
	oracle_set.oracles.push(MultilinearOracleMeta::Committed {
		committed_id: CommittedId {
			batch_id: 0,
			index: 2,
		},
		name: Some("x + y::cout".to_string()),
	});
	oracle_set.oracles.push(MultilinearOracleMeta::Shifted {
		inner_id: OracleId::from(2usize),
		offset: 1,
		block_bits: 5,
		variant: ShiftVariant::LogicalLeft,
		name: Some("x + y::cin".to_string()),
	});
	oracle_set.oracles.push(MultilinearOracleMeta::Committed {
		committed_id: CommittedId {
			batch_id: 0,
			index: 3,
		},
		name: Some("x + y::zout".to_string()),
	});

	let mut table_constraints = vec![];
	table_constraints.push(ConstraintSet {
		n_vars: LOG_SIZE,
		oracle_ids: vec![0, 1, 2, 3, 4],
		constraints: vec![
			Constraint {
				composition: Add(
					Box::from(Add(
						Box::from(Mul(
							Box::from(Add(Box::from(Var(0)), Box::from(Var(3)))),
							Box::from(Add(Box::from(Var(1)), Box::from(Var(3)))),
						)),
						Box::from(Var(3)),
					)),
					Box::from(Var(2)),
				),
				predicate: ConstraintPredicate::Zero,
			},
			Constraint {
				composition: Add(
					Box::from(Add(
						Box::from(Add(Box::from(Var(0)), Box::from(Var(1)))),
						Box::from(Var(3)),
					)),
					Box::from(Var(4)),
				),
				predicate: ConstraintPredicate::Zero,
			},
		],
	});

	let cs = ConstraintSystem::<BinaryField128b> {
		oracles: oracle_set,
		table_constraints: table_constraints,
		non_zero_oracle_ids: vec![],
		flushes: vec![],
		max_channel_id: 0,
	};

	print!("manual_cs: ");
	prove_verify_test(cs, witness).unwrap();

	x + y
}

fn prove_verify_test(
	cs: ConstraintSystem<BinaryField128b>,
	witness: MultilinearExtensionIndex<OptimalUnderlier, BinaryField128b>,
) -> Result<()> {
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

	println!("proving test successful");

	Ok(())
}

fn main() {
	let x = 100u32;
	let y = 200u32;

	let out1 = out_of_circuit(x, y);
	let out2 = cs_builder(x, y);
	assert_eq!(out1, out2);

	let out3 = manual_cs(x, y);
	assert_eq!(out1, out3);
}
