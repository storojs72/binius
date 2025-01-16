// Copyright 2024-2025 Irreducible Inc.

use std::{fmt::Debug, ops::Deref};

use binius_field::{ExtensionField, PackedField, RepackedExtension, TowerField};
use binius_hal::{make_portable_backend, ComputationBackendExt};
use binius_math::{MLEEmbeddingAdapter, MultilinearExtension, MultilinearPoly};
use serde::{Deserialize, Serialize};

use crate::polynomial::{Error, MultivariatePoly};

/// A transparent multilinear polynomial defined as the multilinear extension over a small
/// hypercube.
///
/// Multilinear polynomials are considered transparent if they can be succinctly evaluated. While
/// evaluation of multilinear extensions is generally exponential in the number of variables, when
/// the number of variables is very small, and thus the evaluation hypercube is small, we can
/// consider such a multilinear extension to be transparent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultilinearExtensionTransparent<P, PE, Data = Vec<P>>
where
	P: PackedField,
	PE: PackedField,
	PE::Scalar: ExtensionField<P::Scalar>,
	Data: Deref<Target = [P]>,
{
	data: MLEEmbeddingAdapter<P, PE, Data>,
}

impl<P, PE, Data> MultilinearExtensionTransparent<P, PE, Data>
where
	P: PackedField,
	PE: PackedField,
	PE::Scalar: ExtensionField<P::Scalar>,
	Data: Deref<Target = [P]>,
{
	pub fn from_values(values: Data) -> Result<Self, Error> {
		let mle = MultilinearExtension::from_values_generic(values)?;
		Ok(Self {
			data: mle.specialize(),
		})
	}

	/// Create a new `MultilinearExtensionTransparent` from a set of values and a possibly smaller number of variables.
	pub fn from_values_and_mu(values: Data, n_vars: usize) -> Result<Self, Error> {
		let mle = MultilinearExtension::new(n_vars, values)?;
		Ok(Self {
			data: mle.specialize(),
		})
	}
}

impl<F, P, PE, Data> MultivariatePoly<F> for MultilinearExtensionTransparent<P, PE, Data>
where
	F: TowerField + ExtensionField<P::Scalar>,
	P: PackedField,
	PE: PackedField<Scalar = F> + RepackedExtension<P>,
	Data: Deref<Target = [P]> + Send + Sync + Debug,
{
	fn n_vars(&self) -> usize {
		self.data.n_vars()
	}

	fn degree(&self) -> usize {
		self.data.n_vars()
	}

	fn evaluate(&self, query: &[F]) -> Result<F, Error> {
		// Use the portable CPU backend because the size of the hypercube is small by struct
		// assumption.
		let backend = make_portable_backend();
		let query = backend.multilinear_query(query)?;
		Ok(self.data.evaluate(query.to_ref())?)
	}

	fn binary_tower_level(&self) -> usize {
		F::TOWER_LEVEL - self.data.log_extension_degree()
	}
}

#[cfg(test)]
mod test {
	use std::marker::PhantomData;

	use binius_field::{
		arch::OptimalUnderlier, as_packed_field::PackedType, BinaryField32b, PackedField,
	};
	use binius_math::{MLEEmbeddingAdapter, MultilinearExtension};

	use crate::transparent::MultilinearExtensionTransparent;

	#[test]
	fn test_ser_de() {
		type U = OptimalUnderlier;
		type F = BinaryField32b;
		type PackedF = PackedType<U, F>;

		let me = MultilinearExtension::new(1, vec![PackedF::one()]).unwrap();

		let adapter = MLEEmbeddingAdapter {
			0: me,
			1: PhantomData::<PackedF>::default(),
		};

		let instance = MultilinearExtensionTransparent { data: adapter };

		let bytes = bincode::serialize(&instance).unwrap();
		let de: MultilinearExtensionTransparent<PackedF, _> = bincode::deserialize(&bytes).unwrap();
		assert_eq!(de, instance);
	}
}
