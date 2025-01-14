// Copyright 2024-2025 Irreducible Inc.

use core::iter::IntoIterator;
use std::io::{self, Write, Read};
use std::sync::Arc;
use bytes::BytesMut;

use binius_field::{Field, TowerField};
use binius_math::{ArithExpr, CompositionPolyOS};
use binius_utils::bail;
use itertools::Itertools;
use binius_utils::serialization::{DeserializeBytes, SerializeBytes};

use super::{Error, MultilinearOracleSet, MultilinearPolyOracle, OracleId};

/// Composition trait object that can be used to create lists of compositions of differing
/// concrete types.
pub type TypeErasedComposition<P> = Arc<dyn CompositionPolyOS<P>>;

/// Constraint is a type erased composition along with a predicate on its values on the boolean hypercube
#[derive(Debug, Clone)]
pub struct Constraint<F: Field> {
	pub name: Arc<str>,
	pub composition: ArithExpr<F>,
	pub predicate: ConstraintPredicate<F>,
}

impl<F: Field + SerializeBytes + DeserializeBytes> Constraint<F> {
	pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
		self.composition.write(&mut writer)?;
		self.predicate.write(&mut writer)?;
		Ok(())
	}

	pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
		let composition = ArithExpr::<F>::read(&mut reader)?;
		let predicate = ConstraintPredicate::<F>::read(&mut reader)?;
		Ok(Constraint{
			composition,
			predicate,
		})
	}
}

/// Predicate can either be a sum of values of a composition on the hypercube (sumcheck) or equality to zero
/// on the hypercube (zerocheck)
#[derive(Clone, Debug)]
pub enum ConstraintPredicate<F: Field> {
	Sum(F),
	Zero,
}

impl <F: Field + SerializeBytes + DeserializeBytes> ConstraintPredicate<F> {
	pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
		match self {
			Self::Sum(sum) => {
				writer.write_all(1u32.to_le_bytes().as_slice())?;

				let mut buffer = BytesMut::new();
				sum.serialize_to_bytes(&mut buffer).unwrap();

				writer.write_all(&buffer.to_vec())?;
			},
			Self::Zero => {
				writer.write_all(2u32.to_le_bytes().as_slice())?;
			},
		};
		Ok(())
	}

	pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
		let mut four_bytes_buffer = [0u8; 4];
		reader.read_exact(&mut four_bytes_buffer)?;
		let value = u32::from_le_bytes(four_bytes_buffer);
		let predicate = match value {
			1u32 => {
				let buffer = BytesMut::new();
				let field = F::deserialize_from_bytes(buffer.to_vec().as_slice()).unwrap();
				Self::Sum(field)
			},
			2u32 => {
				Self::Zero
			}
			_ => {
				unreachable!()
			}
		};
		Ok(predicate)
	}
}

/// Constraint set is a group of constraints that operate over the same set of oracle-identified multilinears
#[derive(Debug, Clone)]
pub struct ConstraintSet<F: Field> {
	pub n_vars: usize,
	pub oracle_ids: Vec<OracleId>,
	pub constraints: Vec<Constraint<F>>,
}

impl<F: Field + SerializeBytes + DeserializeBytes> ConstraintSet<F> {
	pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
		// n_vars
		writer.write_all((self.n_vars as u32).to_le_bytes().as_slice())?;

		// oracle_ids
		writer.write_all((self.oracle_ids.len() as u32).to_le_bytes().as_slice())?;
		for oracle in self.oracle_ids.iter() {
			writer.write_all((*oracle as u32).to_le_bytes().as_slice())?;
		};

		// constraints
		writer.write_all((self.constraints.len() as u32).to_le_bytes().as_slice())?;
		for constraint in self.constraints.iter() {
			constraint.write(&mut writer)?;
		}

		Ok(())
	}

	pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
		// n_vars
		let mut n_vars_bytes = [0u8; 4];
		reader.read_exact(&mut n_vars_bytes)?;
		let n_vars = u32::from_le_bytes(n_vars_bytes) as usize;

		// oracle_ids
		let mut oracle_ids_len_bytes = [0u8; 4];
		reader.read_exact(&mut oracle_ids_len_bytes)?;
		let oracle_ids_len = u32::from_le_bytes(oracle_ids_len_bytes) as usize;

		let mut oracle_ids = vec![];
		for _ in 0..oracle_ids_len {
			let mut oracle_id_bytes = [0u8; 4];
			reader.read_exact(&mut oracle_id_bytes)?;
			let oracle_id = u32::from_le_bytes(oracle_id_bytes) as usize;
			oracle_ids.push(oracle_id as OracleId);
		}

		// constraints
		let mut constraints_len_bytes = [0u8; 4];
		reader.read_exact(&mut constraints_len_bytes)?;
		let constraints_len = u32::from_le_bytes(constraints_len_bytes) as usize;

		let mut constraints = vec![];
		for _ in 0..constraints_len {
			constraints.push(Constraint::<F>::read(&mut reader)?);
		}

		Ok(
			ConstraintSet {
				n_vars,
				oracle_ids,
				constraints,
			}
		)
	}
}

// A deferred constraint constructor that instantiates index composition after the superset of oracles is known
#[allow(clippy::type_complexity)]
struct UngroupedConstraint<F: Field> {
	name: Arc<str>,
	oracle_ids: Vec<OracleId>,
	composition: ArithExpr<F>,
	predicate: ConstraintPredicate<F>,
}

/// A builder struct that turns individual compositions over oraclized multilinears into a set of
/// type erased `IndexComposition` instances operating over a superset of oracles of all constraints.
#[derive(Default)]
pub struct ConstraintSetBuilder<F: Field> {
	constraints: Vec<UngroupedConstraint<F>>,
}

impl<F: Field> ConstraintSetBuilder<F> {
	pub fn new() -> Self {
		Self {
			constraints: Vec::new(),
		}
	}

	pub fn add_sumcheck(
		&mut self,
		oracle_ids: impl IntoIterator<Item = OracleId>,
		composition: ArithExpr<F>,
		sum: F,
	) {
		self.constraints.push(UngroupedConstraint {
			name: "sumcheck".into(),
			oracle_ids: oracle_ids.into_iter().collect(),
			composition,
			predicate: ConstraintPredicate::Sum(sum),
		});
	}

	pub fn add_zerocheck(
		&mut self,
		name: impl ToString,
		oracle_ids: impl IntoIterator<Item = OracleId>,
		composition: ArithExpr<F>,
	) {
		self.constraints.push(UngroupedConstraint {
			name: name.to_string().into(),
			oracle_ids: oracle_ids.into_iter().collect(),
			composition,
			predicate: ConstraintPredicate::Zero,
		});
	}

	/// Build a single constraint set, requiring that all included oracle n_vars are the same
	pub fn build_one(
		self,
		oracles: &MultilinearOracleSet<impl TowerField>,
	) -> Result<ConstraintSet<F>, Error> {
		let mut oracle_ids = self
			.constraints
			.iter()
			.flat_map(|constraint| constraint.oracle_ids.clone())
			.collect::<Vec<_>>();
		if oracle_ids.is_empty() {
			// Do not bail!, this error is handled in evalcheck.
			return Err(Error::EmptyConstraintSet);
		}
		for id in oracle_ids.iter() {
			if !oracles.is_valid_oracle_id(*id) {
				bail!(Error::InvalidOracleId(*id));
			}
		}
		oracle_ids.sort();
		oracle_ids.dedup();

		let n_vars = oracle_ids
			.first()
			.map(|id| oracles.n_vars(*id))
			.unwrap_or_default();

		for id in oracle_ids.iter() {
			if oracles.n_vars(*id) != n_vars {
				bail!(Error::ConstraintSetNvarsMismatch {
					expected: n_vars,
					got: oracles.n_vars(*id)
				});
			}
		}

		// at this point the superset of oracles is known and index compositions
		// may be finally instantiated
		let constraints =
			self.constraints
				.into_iter()
				.map(|constraint| Constraint {
					name: constraint.name,
					composition: constraint
						.composition
						.remap_vars(&positions(&constraint.oracle_ids, &oracle_ids).expect(
							"precondition: oracle_ids is a superset of constraint.oracle_ids",
						))
						.expect("Infallible by ConstraintSetBuilder invariants."),
					predicate: constraint.predicate,
				})
				.collect();

		Ok(ConstraintSet {
			n_vars,
			oracle_ids,
			constraints,
		})
	}

	/// Create one ConstraintSet for every unique n_vars used.
	///
	/// Note that you can't mix oracles with different n_vars in a single constraint.
	pub fn build(
		self,
		oracles: &MultilinearOracleSet<impl TowerField>,
	) -> Result<Vec<ConstraintSet<F>>, Error> {
		let connected_oracle_chunks = self
			.constraints
			.iter()
			.map(|constraint| constraint.oracle_ids.clone())
			.chain(oracles.iter().filter_map(|oracle| {
				match oracle {
					MultilinearPolyOracle::Shifted { id, shifted, .. } => {
						Some(vec![id, shifted.inner().id()])
					}
					MultilinearPolyOracle::LinearCombination {
						id,
						linear_combination,
						..
					} => Some(
						linear_combination
							.polys()
							.map(|p| p.id())
							.chain([id])
							.collect(),
					),
					_ => None,
				}
			}))
			.collect::<Vec<_>>();

		let groups = binius_utils::graph::connected_components(
			&connected_oracle_chunks
				.iter()
				.map(|x| x.as_slice())
				.collect::<Vec<_>>(),
		);

		let n_vars_and_constraints = self
			.constraints
			.into_iter()
			.map(|constraint| {
				if constraint.oracle_ids.is_empty() {
					bail!(Error::EmptyConstraintSet);
				}
				for id in constraint.oracle_ids.iter() {
					if !oracles.is_valid_oracle_id(*id) {
						bail!(Error::InvalidOracleId(*id));
					}
				}
				let n_vars = constraint
					.oracle_ids
					.first()
					.map(|id| oracles.n_vars(*id))
					.unwrap();

				for id in constraint.oracle_ids.iter() {
					if oracles.n_vars(*id) != n_vars {
						bail!(Error::ConstraintSetNvarsMismatch {
							expected: n_vars,
							got: oracles.n_vars(*id)
						});
					}
				}
				Ok::<_, Error>((n_vars, constraint))
			})
			.collect::<Result<Vec<_>, _>>()?;

		let grouped_constraints = n_vars_and_constraints
			.into_iter()
			.sorted_by_key(|(_, constraint)| groups[constraint.oracle_ids[0]])
			.chunk_by(|(_, constraint)| groups[constraint.oracle_ids[0]]);

		let constraint_sets = grouped_constraints
			.into_iter()
			.map(|(_, grouped_constraints)| {
				let mut constraints = vec![];
				let mut oracle_ids = vec![];

				let grouped_constraints = grouped_constraints.into_iter().collect::<Vec<_>>();
				let (n_vars, _) = grouped_constraints[0];

				for (_, constraint) in grouped_constraints {
					oracle_ids.extend(&constraint.oracle_ids);
					constraints.push(constraint);
				}
				oracle_ids.sort();
				oracle_ids.dedup();

				let constraints = constraints
					.into_iter()
					.map(|constraint| Constraint {
						name: constraint.name,
						composition: constraint
							.composition
							.remap_vars(&positions(&constraint.oracle_ids, &oracle_ids).expect(
								"precondition: oracle_ids is a superset of constraint.oracle_ids",
							))
							.expect("Infallible by ConstraintSetBuilder invariants."),
						predicate: constraint.predicate,
					})
					.collect();

				ConstraintSet {
					constraints,
					oracle_ids,
					n_vars,
				}
			})
			.collect();

		Ok(constraint_sets)
	}
}

/// Find index of every subset element within the superset.
/// If the superset contains duplicate elements the index of the first match is used
///
/// Returns None if the subset contains elements that don't exist in the superset
fn positions<T: Eq>(subset: &[T], superset: &[T]) -> Option<Vec<usize>> {
	subset
		.iter()
		.map(|subset_item| {
			superset
				.iter()
				.position(|superset_item| superset_item == subset_item)
		})
		.collect()
}

#[allow(dead_code)]
#[allow(clippy::type_complexity)]
fn thunk_acc<F: Field>(
	oracle_ids: Vec<OracleId>,
	composition: ArithExpr<F>,
) -> Box<dyn FnOnce(&[OracleId]) -> ArithExpr<F>> {
	Box::new(move |all_oracle_ids| {
		let indices = oracle_ids
			.iter()
			.map(|subset_item| {
				all_oracle_ids
					.iter()
					.position(|superset_item| superset_item == subset_item)
					.expect("precondition: all_oracle_ids is a superset of oracle_ids")
			})
			.collect::<Vec<usize>>();

		composition
			.remap_vars(&indices)
			.expect("Infallible by ConstraintSetBuilder invariants.")
	})
}
