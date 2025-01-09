// Copyright 2024-2025 Irreducible Inc.

use std::{
	cmp::max,
	fmt::{self, Display},
	iter::{Product, Sum},
	ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};
use std::io::{self, Write, Read};

use binius_field::Field;
use binius_utils::serialization::{ SerializeBytes, DeserializeBytes };

use super::error::Error;

use bytes::BytesMut;

/// Arithmetic expressions that can be evaluated symbolically.
///
/// Arithmetic expressions are trees, where the leaves are either constants or variables, and the
/// non-leaf nodes are arithmetic operations, such as addition, multiplication, etc. They are
/// specific representations of multivariate polynomials.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArithExpr<F: Field> {
	Const(F),
	Var(usize),
	Add(Box<ArithExpr<F>>, Box<ArithExpr<F>>),
	Mul(Box<ArithExpr<F>>, Box<ArithExpr<F>>),
	Pow(Box<ArithExpr<F>>, u64),
}

impl<F: Field + Display> Display for ArithExpr<F> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Const(v) => write!(f, "{v}"),
			Self::Var(i) => write!(f, "x{i}"),
			Self::Add(x, y) => write!(f, "({} + {})", &**x, &**y),
			Self::Mul(x, y) => write!(f, "({} * {})", &**x, &**y),
			Self::Pow(x, p) => write!(f, "({})^{p}", &**x),
		}
	}
}

impl <F: Field + SerializeBytes + DeserializeBytes> ArithExpr<F> {
	pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
		match self {
			Self::Const(constant) => {
				writer.write_all(1u32.to_le_bytes().as_slice())?;

				let mut buffer = BytesMut::new();
				constant.serialize(&mut buffer).unwrap();
				let buffer = buffer.to_vec();
				writer.write_all(buffer.len().to_le_bytes().as_slice())?;
				writer.write_all(&buffer)?;
			}
			Self::Var(variable) => {
				writer.write_all(2u32.to_le_bytes().as_slice())?;

				writer.write_all((*variable as u32).to_le_bytes().as_slice())?;
			}
			Self::Add(addition_left, addition_right) => {
				writer.write_all(3u32.to_le_bytes().as_slice())?;

				let mut buffer = vec![];
				addition_left.write(&mut buffer)?;
				addition_right.write(&mut buffer)?;

				let buffer_len = buffer.len() as u32;
				writer.write_all(buffer_len.to_le_bytes().as_slice())?;
				writer.write_all(buffer.as_slice())?;
			}
			Self::Mul(multiplication_left, multiplication_right) => {
				writer.write_all(4u32.to_le_bytes().as_slice())?;

				let mut buffer = vec![];
				multiplication_left.write(&mut buffer)?;
				multiplication_right.write(&mut buffer)?;

				let buffer_len = buffer.len() as u32;
				writer.write_all(buffer_len.to_le_bytes().as_slice())?;
				writer.write_all(buffer.as_slice())?;
			}
			Self::Pow(power, exponent) => {
				writer.write_all(5u32.to_le_bytes().as_slice())?;

				let mut buffer = vec![];
				power.write(&mut buffer)?;
				buffer.write_all(&exponent.to_le_bytes().as_slice())?;

				let buffer_len = buffer.len() as u32;
				writer.write_all(buffer_len.to_le_bytes().as_slice())?;
				writer.write_all(buffer.as_slice())?;
			}
		}
		Ok(())
	}

	pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
		let mut four_bytes_buffer = [0u8; 4];
		reader.read_exact(&mut four_bytes_buffer)?;
		let value = u32::from_le_bytes(four_bytes_buffer);
		let arith_expr = match value {
			1u32 => {
				let mut four_bytes_buffer = [0u8; 4];
				reader.read_exact(&mut four_bytes_buffer)?;
				let len = u32::from_le_bytes(four_bytes_buffer);

				let mut buffer = BytesMut::zeroed(len as usize);
				reader.read_exact(&mut buffer)?;

				let field = F::deserialize(buffer.to_vec().as_slice()).unwrap();
				Self::Const(field)
			},
			2u32 => {
				let mut four_bytes_buffer = [0u8; 4];
				reader.read_exact(&mut four_bytes_buffer)?;
				let variable = u32::from_le_bytes(four_bytes_buffer);
				Self::Var(variable as usize)
			},
			3u32 => {
				let mut four_bytes_buffer = [0u8; 4];
				reader.read_exact(&mut four_bytes_buffer)?;
				let len = u32::from_le_bytes(four_bytes_buffer);
				let mut buffer = vec![0u8; len as usize];
				reader.read_exact(&mut buffer)?;

				let addition_left = Self::read(buffer.as_slice())?;
				let addition_right = Self::read(buffer.as_slice())?;
				Self::Add(Box::new(addition_left), Box::new(addition_right))
			},
			4u32 => {
				let mut four_bytes_buffer = [0u8; 4];
				reader.read_exact(&mut four_bytes_buffer)?;
				let len = u32::from_le_bytes(four_bytes_buffer);
				let mut buffer = vec![0u8; len as usize];
				reader.read_exact(&mut buffer)?;

				let multiplication_left = Self::read(buffer.as_slice())?;
				let multiplication_right = Self::read(buffer.as_slice())?;
				Self::Mul(Box::new(multiplication_left), Box::new(multiplication_right))
			},
			5u32 => {
				let mut four_bytes_buffer = [0u8; 4];
				reader.read_exact(&mut four_bytes_buffer)?;
				let len = u32::from_le_bytes(four_bytes_buffer);
				let mut buffer = vec![0u8; len as usize];
				reader.read_exact(&mut buffer)?;

				let power = Self::read(buffer.as_slice())?;
				let mut eight_bytes_buffer = [0u8; 8];
				reader.read_exact(&mut eight_bytes_buffer)?;
				let exponent = u64::from_le_bytes(eight_bytes_buffer);
				Self::Pow(Box::new(power), exponent)
			},
			_ => unreachable!("ArithExpr read unreachable"),
		};

		Ok(arith_expr)
	}
}

impl<F: Field> ArithExpr<F> {
	/// The number of variables the expression contains.
	pub fn n_vars(&self) -> usize {
		match self {
			ArithExpr::Const(_) => 0,
			ArithExpr::Var(index) => *index + 1,
			ArithExpr::Add(left, right) | ArithExpr::Mul(left, right) => {
				max(left.n_vars(), right.n_vars())
			}
			ArithExpr::Pow(id, _) => id.n_vars(),
		}
	}

	/// The total degree of the polynomial the expression represents.
	pub fn degree(&self) -> usize {
		match self {
			ArithExpr::Const(_) => 0,
			ArithExpr::Var(_) => 1,
			ArithExpr::Add(left, right) => max(left.degree(), right.degree()),
			ArithExpr::Mul(left, right) => left.degree() + right.degree(),
			ArithExpr::Pow(base, exp) => base.degree() * *exp as usize,
		}
	}

	pub fn pow(self, exp: u64) -> Self {
		ArithExpr::Pow(Box::new(self), exp)
	}

	pub const fn zero() -> Self {
		ArithExpr::Const(F::ZERO)
	}

	pub const fn one() -> Self {
		ArithExpr::Const(F::ONE)
	}

	/// Creates a new expression with the variable indices remapped.
	///
	/// This recursively replaces the variable sub-expressions with an index `i` with the variable
	/// `indices[i]`.
	///
	/// ## Throws
	///
	/// * [`Error::IncorrectArgumentLength`] if indices has length less than the current number of
	///   variables
	pub fn remap_vars(self, indices: &[usize]) -> Result<Self, Error> {
		let expr = match self {
			ArithExpr::Const(_) => self,
			ArithExpr::Var(index) => {
				let new_index =
					indices
						.get(index)
						.ok_or_else(|| Error::IncorrectArgumentLength {
							arg: "subset".to_string(),
							expected: index,
						})?;
				ArithExpr::Var(*new_index)
			}
			ArithExpr::Add(left, right) => {
				let new_left = left.remap_vars(indices)?;
				let new_right = right.remap_vars(indices)?;
				ArithExpr::Add(Box::new(new_left), Box::new(new_right))
			}
			ArithExpr::Mul(left, right) => {
				let new_left = left.remap_vars(indices)?;
				let new_right = right.remap_vars(indices)?;
				ArithExpr::Mul(Box::new(new_left), Box::new(new_right))
			}
			ArithExpr::Pow(base, exp) => {
				let new_base = base.remap_vars(indices)?;
				ArithExpr::Pow(Box::new(new_base), exp)
			}
		};
		Ok(expr)
	}

	pub fn convert_field<FTgt: Field + From<F>>(&self) -> ArithExpr<FTgt> {
		match self {
			ArithExpr::Const(val) => ArithExpr::Const((*val).into()),
			ArithExpr::Var(index) => ArithExpr::Var(*index),
			ArithExpr::Add(left, right) => {
				let new_left = left.convert_field();
				let new_right = right.convert_field();
				ArithExpr::Add(Box::new(new_left), Box::new(new_right))
			}
			ArithExpr::Mul(left, right) => {
				let new_left = left.convert_field();
				let new_right = right.convert_field();
				ArithExpr::Mul(Box::new(new_left), Box::new(new_right))
			}
			ArithExpr::Pow(base, exp) => {
				let new_base = base.convert_field();
				ArithExpr::Pow(Box::new(new_base), *exp)
			}
		}
	}

	pub fn try_convert_field<FTgt: Field + TryFrom<F>>(
		&self,
	) -> Result<ArithExpr<FTgt>, <FTgt as TryFrom<F>>::Error> {
		Ok(match self {
			ArithExpr::Const(val) => ArithExpr::Const((*val).try_into()?),
			ArithExpr::Var(index) => ArithExpr::Var(*index),
			ArithExpr::Add(left, right) => {
				let new_left = left.try_convert_field()?;
				let new_right = right.try_convert_field()?;
				ArithExpr::Add(Box::new(new_left), Box::new(new_right))
			}
			ArithExpr::Mul(left, right) => {
				let new_left = left.try_convert_field()?;
				let new_right = right.try_convert_field()?;
				ArithExpr::Mul(Box::new(new_left), Box::new(new_right))
			}
			ArithExpr::Pow(base, exp) => {
				let new_base = base.try_convert_field()?;
				ArithExpr::Pow(Box::new(new_base), *exp)
			}
		})
	}
}

impl<F> Default for ArithExpr<F>
where
	F: Field,
{
	fn default() -> Self {
		Self::zero()
	}
}

impl<F> Add for ArithExpr<F>
where
	F: Field,
{
	type Output = Self;

	fn add(self, rhs: Self) -> Self {
		ArithExpr::Add(Box::new(self), Box::new(rhs))
	}
}

impl<F> AddAssign for ArithExpr<F>
where
	F: Field,
{
	fn add_assign(&mut self, rhs: Self) {
		*self = std::mem::take(self) + rhs;
	}
}

impl<F> Sub for ArithExpr<F>
where
	F: Field,
{
	type Output = Self;

	fn sub(self, rhs: Self) -> Self {
		ArithExpr::Add(Box::new(self), Box::new(rhs))
	}
}

impl<F> SubAssign for ArithExpr<F>
where
	F: Field,
{
	fn sub_assign(&mut self, rhs: Self) {
		*self = std::mem::take(self) - rhs;
	}
}

impl<F> Mul for ArithExpr<F>
where
	F: Field,
{
	type Output = Self;

	fn mul(self, rhs: Self) -> Self {
		ArithExpr::Mul(Box::new(self), Box::new(rhs))
	}
}

impl<F> MulAssign for ArithExpr<F>
where
	F: Field,
{
	fn mul_assign(&mut self, rhs: Self) {
		*self = std::mem::take(self) * rhs;
	}
}

impl<F: Field> Sum for ArithExpr<F> {
	fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
		iter.reduce(|acc, item| acc + item).unwrap_or(Self::zero())
	}
}

impl<F: Field> Product for ArithExpr<F> {
	fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
		iter.reduce(|acc, item| acc * item).unwrap_or(Self::one())
	}
}

#[cfg(test)]
mod tests {
	use assert_matches::assert_matches;
	use binius_field::{BinaryField128b, BinaryField1b, BinaryField8b};

	use super::*;

	#[test]
	fn test_degree_with_pow() {
		let expr = ArithExpr::Const(BinaryField8b::new(6)).pow(7);
		assert_eq!(expr.degree(), 0);

		let expr: ArithExpr<BinaryField8b> = ArithExpr::Var(0).pow(7);
		assert_eq!(expr.degree(), 7);

		let expr: ArithExpr<BinaryField8b> = (ArithExpr::Var(0) * ArithExpr::Var(1)).pow(7);
		assert_eq!(expr.degree(), 14);
	}

	#[test]
	fn test_remap_vars_with_too_few_vars() {
		type F = BinaryField8b;
		let expr = ((ArithExpr::Var(0) + ArithExpr::Const(F::ONE)) * ArithExpr::Var(1)).pow(3);
		assert_matches!(expr.remap_vars(&[5]), Err(Error::IncorrectArgumentLength { .. }));
	}

	#[test]
	fn test_remap_vars_works() {
		type F = BinaryField8b;
		let expr = ((ArithExpr::Var(0) + ArithExpr::Const(F::ONE)) * ArithExpr::Var(1)).pow(3);
		let new_expr = expr.remap_vars(&[5, 3]);

		let expected = ((ArithExpr::Var(5) + ArithExpr::Const(F::ONE)) * ArithExpr::Var(3)).pow(3);
		assert_eq!(new_expr.unwrap(), expected);
	}

	#[test]
	fn test_expression_upcast() {
		type F8 = BinaryField8b;
		type F = BinaryField128b;

		let expr = ((ArithExpr::Var(0) + ArithExpr::Const(F8::ONE))
			* ArithExpr::Const(F8::new(222)))
		.pow(3);

		let expected =
			((ArithExpr::Var(0) + ArithExpr::Const(F::ONE)) * ArithExpr::Const(F::new(222))).pow(3);
		assert_eq!(expr.convert_field::<F>(), expected);
	}

	#[test]
	fn test_expression_downcast() {
		type F8 = BinaryField8b;
		type F = BinaryField128b;

		let expr =
			((ArithExpr::Var(0) + ArithExpr::Const(F::ONE)) * ArithExpr::Const(F::new(222))).pow(3);

		assert!(expr.clone().try_convert_field::<BinaryField1b>().is_err());

		let expected = ((ArithExpr::Var(0) + ArithExpr::Const(F8::ONE))
			* ArithExpr::Const(F8::new(222)))
		.pow(3);
		assert_eq!(expr.try_convert_field::<BinaryField8b>().unwrap(), expected);
	}
}
