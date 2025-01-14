// Copyright 2024-2025 Irreducible Inc.

use binius_field::{ExtensionField, TowerField};
use binius_utils::bail;

use crate::polynomial::{Error, MultivariatePoly};
//use serde::{Serialize, Serializer};
//use binius_utils::serialization::{DeserializeBytes, SerializeBytes};

/// A constant polynomial.
#[derive(Debug, Copy, Clone)]
pub struct Constant<F> {
	pub n_vars: usize,
	pub value: F,
	pub tower_level: usize,
}

impl<F: TowerField> Constant<F> {
	pub fn new<FS: TowerField>(n_vars: usize, value: FS) -> Self
	where
		F: ExtensionField<FS>,
	{
		Self {
			value: value.into(),
			tower_level: FS::TOWER_LEVEL,
			n_vars,
		}
	}
}

impl<F: TowerField> MultivariatePoly<F> for Constant<F> {
	fn n_vars(&self) -> usize {
		self.n_vars
	}

	fn degree(&self) -> usize {
		0
	}

	fn evaluate(&self, query: &[F]) -> Result<F, Error> {
		if query.len() != self.n_vars {
			bail!(Error::IncorrectQuerySize {
				expected: self.n_vars,
			});
		}
		Ok(self.value)
	}

	fn binary_tower_level(&self) -> usize {
		self.tower_level
	}
}

/*
#[cfg(test)]
mod test {
	use binius_field::{BinaryField128b, Field};
	use crate::transparent::constant::Constant;

	#[test]
	fn test_bincode_serialize() {
		type F = BinaryField128b;
		let c = Constant {
			n_vars: 100usize,
			value: F::ONE,
			tower_level: 1usize,
		};

		bincode::serialize(&c).unwrap();
	}
}*/
