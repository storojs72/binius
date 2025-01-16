// Copyright 2024-2025 Irreducible Inc.

use binius_field::Field;
use binius_utils::bail;
use serde::{Deserialize, Serialize};

use crate::polynomial::{Error, MultivariatePoly};

/// Represents a product of two multilinear polynomials over disjoint variables.
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct DisjointProduct<P0, P1>(pub P0, pub P1);

impl<F: Field, P0, P1> MultivariatePoly<F> for DisjointProduct<P0, P1>
where
	P0: MultivariatePoly<F>,
	P1: MultivariatePoly<F>,
{
	fn n_vars(&self) -> usize {
		self.0.n_vars() + self.1.n_vars()
	}

	fn degree(&self) -> usize {
		self.0.degree() + self.1.degree()
	}

	fn evaluate(&self, query: &[F]) -> Result<F, Error> {
		let p0_vars = self.0.n_vars();
		let p1_vars = self.1.n_vars();
		let n_vars = p0_vars + p1_vars;

		if query.len() != n_vars {
			bail!(Error::IncorrectQuerySize { expected: n_vars });
		}

		let eval0 = self.0.evaluate(&query[..p0_vars])?;
		let eval1 = self.1.evaluate(&query[p0_vars..])?;
		Ok(eval0 * eval1)
	}

	fn binary_tower_level(&self) -> usize {
		self.0.binary_tower_level().max(self.1.binary_tower_level())
	}
}

#[cfg(test)]
mod tests {
	use binius_field::{BinaryField128b, Field};
	use serde_test::{assert_tokens, Token};

	use crate::transparent::{constant::Constant, disjoint_product::DisjointProduct};

	#[test]
	fn test_ser_de() {
		type F = BinaryField128b;
		let one = F::ONE;
		let two = F::from(2u128);

		let c1 = Constant {
			n_vars: 100usize,
			value: one,
			tower_level: 15usize,
		};

		let c2 = Constant {
			n_vars: 200usize,
			value: two,
			tower_level: 30usize,
		};

		let disjoint_product = DisjointProduct(c1, c2);

		assert_tokens(
			&disjoint_product,
			&[
				Token::TupleStruct {
					name: "DisjointProduct",
					len: 2,
				},
				Token::Struct {
					name: "Constant",
					len: 3,
				},
				Token::Str("n_vars"),
				Token::U64(100),
				Token::Str("value"),
				Token::Bytes(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
				Token::Str("tower_level"),
				Token::U64(15),
				Token::StructEnd,
				Token::Struct {
					name: "Constant",
					len: 3,
				},
				Token::Str("n_vars"),
				Token::U64(200),
				Token::Str("value"),
				Token::Bytes(&[2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
				Token::Str("tower_level"),
				Token::U64(30),
				Token::StructEnd,
				Token::TupleStructEnd,
			],
		);

		let bytes = bincode::serialize(&disjoint_product).unwrap();
		let de: DisjointProduct<Constant<F>, Constant<F>> = bincode::deserialize(&bytes).unwrap();

		assert_eq!(de, disjoint_product);
	}
}
