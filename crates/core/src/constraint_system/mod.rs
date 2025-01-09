// Copyright 2024-2025 Irreducible Inc.

pub mod channel;
mod common;
pub mod error;
mod prove;
pub mod validate;
mod verify;

use std::io::{self, Read, Write};
use binius_field::TowerField;
use binius_utils::serialization::{DeserializeBytes, SerializeBytes};
use channel::{ChannelId, Flush};
pub use prove::prove;
pub use verify::verify;

use crate::oracle::{ConstraintSet, MultilinearOracleSet, OracleId};

/// Contains the 3 things that place constraints on witness data in Binius
/// - virtual oracles
/// - polynomial constraints
/// - channel flushes
///
/// As a result, a ConstraintSystem allows us to validate all of these
/// constraints against a witness, as well as enabling generic prove/verify
#[derive(Debug, Clone)]
pub struct ConstraintSystem<F: TowerField> {
	pub oracles: MultilinearOracleSet<F>,
	pub table_constraints: Vec<ConstraintSet<F>>,
	pub non_zero_oracle_ids: Vec<OracleId>,
	pub flushes: Vec<Flush>,
	pub max_channel_id: ChannelId,
}

impl<F: TowerField + SerializeBytes + DeserializeBytes> ConstraintSystem<F> {
	pub fn no_base_constraints(self) -> ConstraintSystem<F> {
		ConstraintSystem {
			oracles: self.oracles,
			table_constraints: self.table_constraints,
			non_zero_oracle_ids: self.non_zero_oracle_ids,
			flushes: self.flushes,
			max_channel_id: self.max_channel_id,
		}
	}

	pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
		// oracles
		self.oracles.write(&mut writer)?;

		// table_constraints
		writer.write_all((self.table_constraints.len() as u32).to_le_bytes().as_slice())?;
		for constraint_set in self.table_constraints.iter() {
			constraint_set.write(&mut writer)?;
		}

		// non_zero_oracle_ids
		writer.write_all((self.non_zero_oracle_ids.len() as u32).to_le_bytes().as_slice())?;
		for oracle in self.non_zero_oracle_ids.iter() {
			writer.write_all((*oracle as u32).to_le_bytes().as_slice())?;
		}

		// flushes
		writer.write_all((self.flushes.len() as u32).to_le_bytes().as_slice())?;
		for flush in self.flushes.iter() {
			flush.write(&mut writer)?;
		}

		// max_channel_id
		writer.write_all((self.max_channel_id as u32).to_le_bytes().as_slice())?;


		Ok(())
	}

	pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
		// oracles
		let oracles = MultilinearOracleSet::<F>::read(&mut reader)?;

		// table_constraints
		let mut table_constraints_len_bytes = [0u8; 4];
		reader.read_exact(&mut table_constraints_len_bytes)?;
		let table_constraints_len = u32::from_le_bytes(table_constraints_len_bytes);

		let mut table_constraints = vec![];
		for _ in 0..table_constraints_len {
			let constraint_set = ConstraintSet::<F>::read(&mut reader)?;
			table_constraints.push(constraint_set);
		}

		// non_zero_oracle_ids
		let mut non_zero_oracle_ids_len_bytes = [0u8; 4];
		reader.read_exact(&mut non_zero_oracle_ids_len_bytes)?;
		let non_zero_oracle_ids_len = u32::from_le_bytes(non_zero_oracle_ids_len_bytes);

		let mut non_zero_oracle_ids = vec![];
		for _ in 0..non_zero_oracle_ids_len {
			let mut non_zero_oracle_id_bytes = [0u8; 4];
			reader.read_exact(&mut non_zero_oracle_id_bytes)?;
			let non_zero_oracle_id = u32::from_le_bytes(non_zero_oracle_id_bytes) as usize;
			non_zero_oracle_ids.push(non_zero_oracle_id as OracleId);
		}

		// flushes
		let mut flushes_len_bytes = [0u8; 4];
		reader.read_exact(&mut flushes_len_bytes)?;
		let flushes_len = u32::from_le_bytes(flushes_len_bytes);

		let mut flushes = vec![];
		for _ in 0..flushes_len {
			let flush = Flush::read(&mut reader)?;
			flushes.push(flush);
		}

		// max_channel_id
		let mut max_channel_id_bytes = [0u8; 4];
		reader.read_exact(&mut max_channel_id_bytes)?;
		let max_channel_id = u32::from_le_bytes(max_channel_id_bytes) as usize;

		Ok(ConstraintSystem{
			oracles,
			table_constraints,
			non_zero_oracle_ids,
			flushes,
			max_channel_id,
		})
	}
}

/// Constraint system proof that has been serialized into bytes
#[derive(Debug, Clone)]
pub struct Proof {
	pub transcript: Vec<u8>,
}

impl Proof {
	pub fn get_proof_size(&self) -> usize {
		self.transcript.len()
	}

	pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()>{
		assert!(self.transcript.len() < u32::MAX as usize, "too long transcript");
		assert!(self.advice.len() < u32::MAX as usize, "too long advice");

		writer.write_all((self.transcript.len() as u32).to_le_bytes().as_slice())?;
		writer.write_all(self.transcript.as_slice())?;

		writer.write_all((self.advice.len() as u32).to_le_bytes().as_slice())?;
		writer.write_all(self.advice.as_slice())?;
		Ok(())
	}

	pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
		let mut transcript_len: [u8; 4] = [0u8; 4];
		reader.read_exact(&mut transcript_len)?;
		let mut transcript = vec![0u8; u32::from_le_bytes(transcript_len) as usize];
		reader.read_exact(&mut transcript)?;

		let mut advice_len: [u8; 4] = [0u8; 4];
		reader.read_exact(&mut advice_len)?;
		let mut advice = vec![0u8; u32::from_le_bytes(advice_len) as usize];
		reader.read_exact(&mut advice)?;

		Ok(Proof {
			transcript,
			advice
		})
	}
}
