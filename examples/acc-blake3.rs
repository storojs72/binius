use anyhow::Result;
use binius_circuits::{
	bitwise::xor, builder::ConstraintSystemBuilder, sha256, u32add::u32add_committed,
	unconstrained::variable,
};
use binius_core::{
	constraint_system, fiat_shamir::HasherChallenger, oracle::OracleId, tower::CanonicalTowerFamily,
};
use binius_field::{
	arch::OptimalUnderlier, as_packed_field::PackScalar, underlier::UnderlierType, BinaryField128b,
	BinaryField1b, BinaryField8b, TowerField,
};
use binius_hal::make_portable_backend;
use binius_hash::{GroestlDigestCompression, GroestlHasher};
use binius_math::DefaultEvaluationDomainFactory;
use binius_utils::checked_arithmetics::log2_ceil_usize;
use bytemuck::Pod;
use groestl_crypto::Groestl256;

const COMPRESSION_LOG_LEN: usize = 5;

// The Blake3 mixing function, G, which mixes either a column or a diagonal.
// https://github.com/BLAKE3-team/BLAKE3/blob/master/reference_impl/reference_impl.rs
fn g(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize, mx: u32, my: u32) {
	state[a] = state[a].wrapping_add(state[b]).wrapping_add(mx);
	state[d] = (state[d] ^ state[a]).rotate_right(16);
	state[c] = state[c].wrapping_add(state[d]);
	state[b] = (state[b] ^ state[c]).rotate_right(12);
	state[a] = state[a].wrapping_add(state[b]).wrapping_add(my);
	state[d] = (state[d] ^ state[a]).rotate_right(8);
	state[c] = state[c].wrapping_add(state[d]);
	state[b] = (state[b] ^ state[c]).rotate_right(7);
}

fn round(state: &mut [u32; 16], m: &[u32; 16]) {
	// Mix the columns.
	g(state, 0, 4, 8, 12, m[0], m[1]);
	g(state, 1, 5, 9, 13, m[2], m[3]);
	g(state, 2, 6, 10, 14, m[4], m[5]);
	g(state, 3, 7, 11, 15, m[6], m[7]);
	// Mix the diagonals.
	g(state, 0, 5, 10, 15, m[8], m[9]);
	g(state, 1, 6, 11, 12, m[10], m[11]);
	g(state, 2, 7, 8, 13, m[12], m[13]);
	g(state, 3, 4, 9, 14, m[14], m[15]);
}

const IV: [u32; 8] = [
	0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

const MSG_PERMUTATION: [usize; 16] = [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8];

fn permute(m: &mut [u32; 16]) {
	let mut permuted = [0; 16];
	for i in 0..16 {
		permuted[i] = m[MSG_PERMUTATION[i]];
	}
	*m = permuted
}

fn compress(
	chaining_value: &[u32; 8],
	block_words: &[u32; 16],
	counter: u64,
	block_len: u32,
	flags: u32,
) -> [u32; 16] {
	let counter_low = counter as u32;
	let counter_high = (counter >> 32) as u32;
	let mut state = [
		chaining_value[0],
		chaining_value[1],
		chaining_value[2],
		chaining_value[3],
		chaining_value[4],
		chaining_value[5],
		chaining_value[6],
		chaining_value[7],
		IV[0],
		IV[1],
		IV[2],
		IV[3],
		counter_low,
		counter_high,
		block_len,
		flags,
	];

	let mut block = *block_words;

	round(&mut state, &block); // round 1
	permute(&mut block);
	round(&mut state, &block); // round 2
	permute(&mut block);
	round(&mut state, &block); // round 3
	permute(&mut block);
	round(&mut state, &block); // round 4
	permute(&mut block);
	round(&mut state, &block); // round 5
	permute(&mut block);
	round(&mut state, &block); // round 6
	permute(&mut block);
	round(&mut state, &block); // round 7
	permute(&mut block);

	for i in 0..8 {
		state[i] ^= state[i + 8];
		state[i + 8] ^= chaining_value[i];
	}
	state
}

fn out_of_circuit_computation() -> Vec<u32> {
	let chaining_value = [0u32; 8];
	let block_words = [0u32; 16];
	let counter = u64::MAX;
	let block_len = u32::MAX;
	let flags = u32::MAX;

	let state = compress(&chaining_value, &block_words, counter, block_len, flags);

	state.to_vec()
}

fn in_circuit_computation() -> Result<Vec<u32>> {
	fn in_circuit_permute(m: &mut [OracleId; 16]) {
		let mut permuted = [OracleId::default(); 16];
		for i in 0..16 {
			permuted[i] = m[MSG_PERMUTATION[i]];
		}
		*m = permuted
	}
	fn in_circuit_g(
		builder: &mut ConstraintSystemBuilder<U, BinaryField128b>,
		state: &mut [OracleId; 16],
		a: usize,
		b: usize,
		c: usize,
		d: usize,
		mx: OracleId,
		my: OracleId,
	) -> Result<()> {
		let n_compressions = 32;
		let log_size = log2_ceil_usize(n_compressions) + COMPRESSION_LOG_LEN;

		let a_add_b = u32add_committed(
			builder,
			format!("a + b, oracle_id: {}, oracle_id: {}", state[a], state[b]),
			state[a],
			state[b],
		)?;
		let state_a = u32add_committed(
			builder,
			format!("a + b + mx, oracle_id: {}, oracle_id: {}", a_add_b, mx),
			a_add_b,
			mx,
		)?;
		let d_xor_a = xor(
			builder,
			format!("d ^ a, oracle_id: {}, oracle_id: {}", state[d], state_a),
			state[d],
			state_a,
		)?;

		// TODO write custom 'rotate' function for this example
		let state_d = sha256::rotate_and_xor(
			log_size,
			builder,
			&[(d_xor_a, 16, sha256::RotateRightType::Circular)],
		)?;

		let state_c = u32add_committed(
			builder,
			format!("c + d, oracle_id: {}, oracle_id: {}", state[c], state_d),
			state[c],
			state_d,
		)?;
		let b_xor_c = xor(
			builder,
			format!("b ^ c, oracle_id: {}, oracle_id: {}", state[b], state_c),
			state[b],
			state_c,
		)?;

		// TODO write custom 'rotate' function for this example
		let state_b = sha256::rotate_and_xor(
			log_size,
			builder,
			&[(b_xor_c, 12, sha256::RotateRightType::Circular)],
		)?;

		let state_a = u32add_committed(
			builder,
			format!("a + b, oracle_id: {}, oracle_id: {}", state_a, state_b),
			state_a,
			state_b,
		)?;
		let state_a = u32add_committed(
			builder,
			format!("a + b + my, oracle_id: {}, oracle_id: {}", state_a, my),
			state_a,
			my,
		)?;
		let d_xor_a = xor(
			builder,
			format!("d ^ a, oracle_id: {}, oracle_id: {}", state_d, state_a),
			state_d,
			state_a,
		)?;

		// TODO write custom 'rotate' function for this example
		let state_d = sha256::rotate_and_xor(
			log_size,
			builder,
			&[(d_xor_a, 8, sha256::RotateRightType::Circular)],
		)?;

		let state_c = u32add_committed(
			builder,
			format!("c + d, oracle_id: {}, oracle_id: {}", state_c, state_d),
			state_c,
			state_d,
		)?;

		let b_xor_c = xor(
			builder,
			format!("b ^ c, oracle_id: {}, oracle_id: {}", state_b, state_c),
			state_b,
			state_c,
		)?;
		let state_b = sha256::rotate_and_xor(
			log_size,
			builder,
			&[(b_xor_c, 7, sha256::RotateRightType::Circular)],
		)?;

		state[a] = state_a;
		state[b] = state_b;
		state[c] = state_c;
		state[d] = state_d;

		Ok(())
	}
	fn in_circuit_round(
		builder: &mut ConstraintSystemBuilder<U, BinaryField128b>,
		state: &mut [OracleId; 16],
		m: &mut [OracleId; 16],
	) -> Result<()> {
		// Mix the columns
		in_circuit_g(builder, state, 1, 5, 9, 13, m[2], m[3])?;
		in_circuit_g(builder, state, 2, 6, 10, 14, m[4], m[5])?;
		in_circuit_g(builder, state, 3, 7, 11, 15, m[6], m[7])?;
		in_circuit_g(builder, state, 0, 4, 8, 12, m[0], m[1])?;

		// Mix the diagonals
		in_circuit_g(builder, state, 0, 5, 10, 15, m[8], m[9])?;
		in_circuit_g(builder, state, 1, 6, 11, 12, m[10], m[11])?;
		in_circuit_g(builder, state, 2, 7, 8, 13, m[12], m[13])?;
		in_circuit_g(builder, state, 3, 4, 9, 14, m[14], m[15])?;

		Ok(())
	}

	let n_compressions = 32;
	let log_size = log2_ceil_usize(n_compressions) + COMPRESSION_LOG_LEN;

	type U = OptimalUnderlier;
	let allocator = bumpalo::Bump::new();
	let mut builder = ConstraintSystemBuilder::<U, BinaryField128b>::new_with_witness(&allocator);

	let chaining_value =
		[variable::<_, _, BinaryField1b>(&mut builder, "preimage", log_size, 0u32).unwrap(); 8];
	let block_words =
		[variable::<_, _, BinaryField1b>(&mut builder, "block_words", log_size, 0u32).unwrap(); 16];
	let counter_low =
		variable::<_, _, BinaryField1b>(&mut builder, "counter_low", log_size, u64::MAX as u32)
			.unwrap();
	let counter_high = variable::<_, _, BinaryField1b>(
		&mut builder,
		"counter_high",
		log_size,
		(u64::MAX >> 32) as u32,
	)
	.unwrap();
	let block_len =
		variable::<_, _, BinaryField1b>(&mut builder, "block_len", log_size, u32::MAX).unwrap();
	let flags = variable::<_, _, BinaryField1b>(&mut builder, "flags", log_size, u32::MAX).unwrap();

	let mut state = [
		chaining_value[0],
		chaining_value[1],
		chaining_value[2],
		chaining_value[3],
		chaining_value[4],
		chaining_value[5],
		chaining_value[6],
		chaining_value[7],
		variable::<_, _, BinaryField1b>(&mut builder, "iv0", log_size, IV[0]).unwrap(),
		variable::<_, _, BinaryField1b>(&mut builder, "iv1", log_size, IV[1]).unwrap(),
		variable::<_, _, BinaryField1b>(&mut builder, "iv2", log_size, IV[2]).unwrap(),
		variable::<_, _, BinaryField1b>(&mut builder, "iv3", log_size, IV[3]).unwrap(),
		counter_low,
		counter_high,
		block_len,
		flags,
	];

	let mut m = block_words;

	in_circuit_round(&mut builder, &mut state, &mut m)?;
	in_circuit_permute(&mut m);
	in_circuit_round(&mut builder, &mut state, &mut m)?;
	in_circuit_permute(&mut m);
	in_circuit_round(&mut builder, &mut state, &mut m)?;
	in_circuit_permute(&mut m);
	in_circuit_round(&mut builder, &mut state, &mut m)?;
	in_circuit_permute(&mut m);
	in_circuit_round(&mut builder, &mut state, &mut m)?;
	in_circuit_permute(&mut m);
	in_circuit_round(&mut builder, &mut state, &mut m)?;
	in_circuit_permute(&mut m);
	in_circuit_round(&mut builder, &mut state, &mut m)?;
	in_circuit_permute(&mut m);

	for i in 0..8 {
		state[i] =
			xor(&mut builder, format!("state[{}] ^= state[{} + 8]", i, i), state[i], state[i + 8])?;
		state[i + 8] = xor(
			&mut builder,
			format!("state[{} + 8] ^= chaining_value[{}]", i, i),
			state[i + 8],
			chaining_value[i],
		)?;
	}

	// get computed state (e.g. public values)
	let mut state_vector = vec![];
	for item_id in state {
		state_vector.push(get_u32_by_id::<U, BinaryField128b>(&mut builder, item_id));
	}

	prove_verify_test(builder)?;

	Ok(state_vector)
}

fn get_u32_by_id<U, F>(builder: &mut ConstraintSystemBuilder<U, F>, id: OracleId) -> u32
where
	U: UnderlierType + Pod + PackScalar<F> + PackScalar<BinaryField1b>,
	F: TowerField,
{
	let witness = builder.witness().unwrap();

	let val = witness.get::<BinaryField1b>(id).unwrap().as_slice::<u8>();
	u32::from_le_bytes(val[0..4].try_into().unwrap())
}

fn prove_verify_test(
	mut builder: ConstraintSystemBuilder<OptimalUnderlier, BinaryField128b>,
) -> Result<()> {
	let witness = builder
		.take_witness()
		.expect("builder created with witness");

	let constraint_system = builder.build()?;

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
	>(&constraint_system, 1usize, 100usize, witness, &domain_factory, &backend)?;

	constraint_system::verify::<
		OptimalUnderlier,
		CanonicalTowerFamily,
		_,
		_,
		GroestlHasher<BinaryField128b>,
		GroestlDigestCompression<BinaryField8b>,
		HasherChallenger<Groestl256>,
	>(&constraint_system, 1usize, 100usize, &domain_factory, vec![], proof)?;

	println!("proving test successful");

	Ok(())
}

fn main() -> Result<()> {
	let a = out_of_circuit_computation();
	let b = in_circuit_computation().unwrap();
	assert_eq!(a, b);
	Ok(())
}
