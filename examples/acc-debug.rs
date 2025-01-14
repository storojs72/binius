use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;
use binius_field::{BinaryField128b, BinaryField1b, BinaryField64b, BinaryField8b, ExtensionField, Field, PackedBinaryField16x8b, PackedBinaryField1x128b, PackedBinaryField8x16b, PackedField, TowerField};
use serde::{Serialize, Deserialize, Serializer};
use serde::ser::SerializeStruct;
use binius_core::oracle::ShiftVariant;
use binius_field::arch::{OptimalUnderlier, OptimalUnderlier256b};
use binius_field::as_packed_field::PackedType;
use binius_utils::serialization::SerializeBytes;


#[derive(Debug, Serialize)]
pub struct TransparentPolyOracle<F, FExt> where
	F: Field + TowerField + Debug + Send + Sync,
	FExt: ExtensionField<F>,
{
	poly: Arc<MultivariatePoly<F, FExt>>,
}

#[derive(Debug, Clone, Serialize)]
enum MultivariatePoly<F, FExt>
where
	F: Field + TowerField + Debug + Send + Sync,
	FExt: ExtensionField<F>,
{
	Constant { val: Constant<F> },
	DisjointProduct { val: DisjointProduct<Box<MultivariatePoly<F, FExt>>, Box<MultivariatePoly<F, FExt>>> },
	RingSwitchEqInd { val: RingSwitchEqInd<F, FExt>},
	SelectRow { val: SelectRow },
	ShiftIndPartialEval { val: ShiftIndPartialEval<F> },
	StepDown { val: StepDown },
	StepUp { val: StepUp },
	TowerBasis { val: TowerBasis<F> },
	EqIndPartialEval { val: EqIndPartialEval<F> },
	Powers { val: Powers<F> }
	// TODO: MultilinearExtensionTransparent
}


#[derive(Debug, Clone, Serialize)]
pub struct Constant<F: TowerField> {
	pub n_vars: usize,
	pub value: F,
	pub tower_level: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelectRow {
	n_vars: usize,
	index: usize,
}


#[derive(Debug, Clone, Serialize)]
pub struct DisjointProduct<P0, P1>(pub P0, pub P1);


#[derive(Debug, Clone, Serialize)]
pub struct ShiftIndPartialEval<F: Field> {
	block_size: usize,
	shift_offset: usize,
	shift_variant: ShiftVariant,
	r: Vec<F>,
}


#[derive(Debug, Clone, Serialize)]
pub struct RingSwitchEqInd<F: Field, FExt: ExtensionField<F>> {
	z_vals: Arc<[FExt]>,
	row_batch_coeffs: Arc<[FExt]>,
	mixing_coeff: FExt,
	_marker: PhantomData<F>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TowerBasis<F: Field> {
	k: usize,
	iota: usize,
	_marker: PhantomData<F>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepDown {
	n_vars: usize,
	index: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepUp {
	n_vars: usize,
	index: usize,
}


#[derive(Debug, Clone, Serialize)]
pub struct EqIndPartialEval<F: Field> {
	n_vars: usize,
	r: Vec<F>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Powers<F: Field> {
	n_vars: usize,
	base: F,
}

impl<F, FExt> MultivariatePoly<F, FExt>
where
	F: Field + TowerField + Debug + Send + Sync,
	FExt: ExtensionField<F>,
{
	fn do_something(&self) {
		match *self {
			MultivariatePoly::Constant { ref val, } => println!("I'm a Constant and this is my data {:?}", val),
			MultivariatePoly::DisjointProduct {ref val, } => println!("I'm Disjoint product and this is my data: {:?}", val),
			MultivariatePoly::RingSwitchEqInd { ref val, } => println!("I'm RingSwitchEqInd and this is my data: {:?}", val),
			MultivariatePoly::SelectRow { ref val, } => {
				println!("I'm SelectRow and this is my data {:?}", val)
			}
			MultivariatePoly::ShiftIndPartialEval {ref val, } => println!("I'm ShiftIndPartialEval and this is my data: {:?}", val),
			MultivariatePoly::StepDown { ref val, } => println!("I'm StepDown and this is my data: {:?}", val),
			MultivariatePoly::StepUp { ref val, } => println!("I'm StepUp and this is my data: {:?}", val),
			MultivariatePoly::TowerBasis {ref val, } => println!("I'm TowerBasis and this is my data: {:?}", val),
			MultivariatePoly::EqIndPartialEval { ref val, } => println!("I'm EqIndPartialEval ad this is my data: {:?}", val),
			MultivariatePoly::Powers {ref val, } => println!("I'm Powers and this is my data: {:?}", val),
		}
	}
}

fn main() {
	type F = BinaryField8b;
	type FExt = BinaryField128b;

	let one = F::ONE;
	let one_ext = FExt::ONE;

	// constant
	let constant: MultivariatePoly<F, FExt> = MultivariatePoly::Constant { val: Constant { n_vars: 10usize, value: one, tower_level: 10usize } };
	constant.do_something();
	let _bytes = bincode::serialize(&constant).unwrap();


	let transparent_poly_oracle = TransparentPolyOracle::<F, FExt> {
		poly: Arc::new(constant.clone())
	};
	let _bytes = bincode::serialize(&transparent_poly_oracle).unwrap();


	// select row
	let select_row: MultivariatePoly<F, FExt> = MultivariatePoly::SelectRow { val: SelectRow { n_vars: 20usize, index: 20usize } };
	select_row.do_something();
	let _bytes = bincode::serialize(&select_row).unwrap();

	let transparent_poly_oracle = TransparentPolyOracle::<F, FExt> {
		poly: Arc::new(select_row.clone())
	};
	let _bytes = bincode::serialize(&transparent_poly_oracle).unwrap();

	// disjoint product
	let disjoint_product: MultivariatePoly<F, FExt> = MultivariatePoly::DisjointProduct { val: DisjointProduct { 0: Box::new(constant), 1: Box::new(select_row) } };
	disjoint_product.do_something();
	let _bytes = bincode::serialize(&disjoint_product).unwrap();

	let transparent_poly_oracle = TransparentPolyOracle::<F, FExt> {
		poly: Arc::new(disjoint_product)
	};
	let _bytes = bincode::serialize(&transparent_poly_oracle).unwrap();

	// ShiftIndPartialEval
	let shift_ind_partial_eval = MultivariatePoly::<F, FExt>::ShiftIndPartialEval {
		val: ShiftIndPartialEval {
			block_size: 0usize,
			shift_offset: 0usize,
			shift_variant: ShiftVariant::CircularLeft,
			r: vec![one, one]
		}
	};

	shift_ind_partial_eval.do_something();

	let _bytes = bincode::serialize(&shift_ind_partial_eval).unwrap();

	let transparent_poly_oracle = TransparentPolyOracle::<F, FExt> {
		poly: Arc::new(shift_ind_partial_eval)
	};
	let _bytes = bincode::serialize(&transparent_poly_oracle).unwrap();

	// RingSwitchEqInd
	let ring_switch_eq_ind = MultivariatePoly::<F, FExt>::RingSwitchEqInd {
		val: RingSwitchEqInd {
			z_vals: Arc::new([one_ext]),
			row_batch_coeffs: Arc::new([one_ext]),
			mixing_coeff: one_ext,
			_marker: Default::default()
		}
	};

	ring_switch_eq_ind.do_something();

	let _bytes = bincode::serialize(&ring_switch_eq_ind).unwrap();

	let transparent_poly_oracle = TransparentPolyOracle {
		poly: Arc::new(ring_switch_eq_ind)
	};

	let _bytes = bincode::serialize(&transparent_poly_oracle).unwrap();

	// StepDown
	let step_down = MultivariatePoly::<F, FExt>::StepDown {
		val: StepDown {
			n_vars: 0usize,
			index: 0usize,
		}
	};

	step_down.do_something();

	let _bytes = bincode::serialize(&step_down).unwrap();

	let transparent_poly_oracle = TransparentPolyOracle {
		poly: Arc::new(step_down.clone())
	};

	let _bytes = bincode::serialize(&step_down).unwrap();


	// StepUp
	let step_up = MultivariatePoly::<F, FExt>::StepUp {
		val: StepUp {
			n_vars: 0usize,
			index: 0usize,
		}
	};

	step_up.do_something();

	let _bytes = bincode::serialize(&step_down).unwrap();

	let transparent_poly_oracle = TransparentPolyOracle {
		poly: Arc::new(step_up.clone())
	};

	let _bytes = bincode::serialize(&transparent_poly_oracle).unwrap();

	// TowerBasis
	let tower_basis = MultivariatePoly::<F, FExt>::TowerBasis {
		val: TowerBasis {
			k: 10usize,
			iota: 20usize,
			_marker: Default::default()
		}
	};
	tower_basis.do_something();

	let _bytes = bincode::serialize(&tower_basis).unwrap();

	let transparent_poly_oracle = TransparentPolyOracle {
		poly: Arc::new(tower_basis)
	};

	let _bytes = bincode::serialize(&transparent_poly_oracle).unwrap();


	// EqIndPartialEval
	let eq_ind_partial_eval = MultivariatePoly::<F, FExt>::EqIndPartialEval {
		val: EqIndPartialEval {
			n_vars: 0usize,
			r: vec![one],
		}
	};

	eq_ind_partial_eval.do_something();

	let _bytes = bincode::serialize(&eq_ind_partial_eval).unwrap();

	let transparent_poly_oracle = TransparentPolyOracle {
		poly: Arc::new(eq_ind_partial_eval.clone())
	};

	let _bytes = bincode::serialize(&transparent_poly_oracle).unwrap();


	// Powers
	let powers = MultivariatePoly::<F, FExt>::Powers {
		val: Powers {
			n_vars: 0usize,
			base: one
		}
	};

	powers.do_something();

	let _bytes = bincode::serialize(&powers).unwrap();

	let transparent_poly_oracle = TransparentPolyOracle {
		poly: Arc::new(powers.clone())
	};

	let _bytes = bincode::serialize(&transparent_poly_oracle).unwrap();
}

// Serializing generic arrays https://github.com/serde-rs/serde/issues/1937
