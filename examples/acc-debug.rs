/*use std::fmt::Debug;
use serde::Deserialize;
use binius_field::{BinaryField128b, Field};


#[typetag::deserialize(tag = "driver")]
trait Contract: Debug + Send + Sync {
	fn do_something(&self);
}

#[derive(Debug, Deserialize, PartialEq)]
struct File {
	path: String,
}

#[typetag::deserialize(name = "file")]
impl Contract for File {
	fn do_something(&self) {
		eprintln!("I'm a File {}", self.path);
	}
}

#[derive(Debug, Deserialize, PartialEq)]
struct Http {
	port: u16,
	endpoint: String,
}

#[typetag::deserialize(name = "http")]
impl Contract for Http {
	fn do_something(&self) {
		eprintln!("I'm an Http {}:{}", self.endpoint, self.port);
	}
}

fn main() {
	let f = r#"
{
  "driver": "file",
  "path": "/var/log/foo"
}
"#;

	let h = r#"
{
  "driver": "http",
  "port": 8080,
  "endpoint": "/api/bar"
}
"#;

	let f: Box<dyn Contract> = serde_json::from_str(f).unwrap();
	f.do_something();

	let h: Box<dyn Contract> = serde_json::from_str(h).unwrap();
	h.do_something();
}
*/


use std::borrow::Borrow;
use std::sync::Arc;
use binius_field::{BinaryField128b, BinaryField64b, Field};
use serde::{Serialize, Deserialize, Serializer};
use serde::ser::SerializeStruct;
use binius_utils::serialization::SerializeBytes;

#[derive(Debug, Serialize)]
pub struct TransparentPolyOracle<F: Field> {
	poly: Arc<MultivariatePoly<F>>,
}

#[derive(Debug, Serialize)]
enum MultivariatePoly<F> {
	Constant { data: Vec<Constant<F>> },
	SelectRow { data: Vec<SelectRow> },
}

#[derive(Debug, Serialize)]
pub struct Constant<F> {
	pub n_vars: usize,
	pub value: F,
	pub tower_level: usize,
}

#[derive(Debug, Serialize)]
pub struct SelectRow {
	n_vars: usize,
	index: usize,
}

impl<F: Field> MultivariatePoly<F> {
	fn do_something(&self) {
		match *self {
			MultivariatePoly::Constant { ref data } => println!("I'm a Constant and this is my data {:?}", data),
			MultivariatePoly::SelectRow { ref data } => {
				println!("I'm SelectRow and this is my data {:?}", data)
			}
		}
	}
}

fn main() {
	type F = BinaryField128b;
	let one = F::ONE;
	let data: MultivariatePoly<F> = MultivariatePoly::Constant { data: vec![ Constant { n_vars: 10usize, value: one, tower_level: 10usize } ] };
	data.do_something();
	let _bytes = bincode::serialize(&data).unwrap();

	let transparent_poly_oracle = TransparentPolyOracle::<F> {
		poly: Arc::new(data)
	};
	println!("transparent_poly_oracle: {:?}", transparent_poly_oracle);
	let _bytes = bincode::serialize(&transparent_poly_oracle).unwrap();


	let data: MultivariatePoly<F> = MultivariatePoly::SelectRow { data: vec![ SelectRow { n_vars: 20usize, index: 20usize } ]};
	data.do_something();
	let _bytes = bincode::serialize(&data).unwrap();

	let transparent_poly_oracle = TransparentPolyOracle {
		poly: Arc::new(data)
	};
	println!("transparent_poly_oracle: {:?}", transparent_poly_oracle);
	let _bytes = bincode::serialize(&transparent_poly_oracle).unwrap();
}
