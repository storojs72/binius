use std::time::Instant;
use binius_core::{
	constraint_system, constraint_system::ConstraintSystem, fiat_shamir::HasherChallenger,
	tower::CanonicalTowerFamily, witness::MultilinearExtensionIndex,
};
use binius_field::{arch::OptimalUnderlier, BinaryField128b};
use binius_hal::make_portable_backend;
use binius_hash::compress::Groestl256ByteCompression;
use binius_math::DefaultEvaluationDomainFactory;
use groestl_crypto::Groestl256;

pub fn prove_verify_test(
	witness: MultilinearExtensionIndex<OptimalUnderlier, BinaryField128b>,
	constraints: ConstraintSystem<BinaryField128b>,
) -> (bool, bool) {
	let domain_factory = DefaultEvaluationDomainFactory::default();
	let backend = make_portable_backend();

	let start = Instant::now();
	let proof = constraint_system::prove::<
		OptimalUnderlier,
		CanonicalTowerFamily,
		_,
		Groestl256,
		Groestl256ByteCompression,
		HasherChallenger<Groestl256>,
		_,
	>(&constraints, 1usize, 100usize, witness, &domain_factory, &backend);
	println!("proving time: {:?}", start.elapsed().as_millis());

	let prove_no_issues = proof.is_ok();
	if !prove_no_issues {
		// Since we have issue on proving, verification is also an issue
		return (false, false);
	}
	let proof = proof.unwrap();
	println!("proof size: {:?}", proof.get_proof_size());
	let start = Instant::now();
	let out = constraint_system::verify::<
		OptimalUnderlier,
		CanonicalTowerFamily,
		Groestl256,
		Groestl256ByteCompression,
		HasherChallenger<Groestl256>,
	>(&constraints, 1usize, 100usize, vec![], proof);
	println!("verification time: {:?}", start.elapsed().as_millis());


	let verify_no_issues = out.is_ok();
	(prove_no_issues, verify_no_issues)
}
