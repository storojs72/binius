use binius_core::{
    constraint_system, constraint_system::ConstraintSystem, fiat_shamir::HasherChallenger,
    tower::CanonicalTowerFamily, witness::MultilinearExtensionIndex,
};
use binius_field::{arch::OptimalUnderlier, BinaryField128b, BinaryField8b};
use binius_hal::make_portable_backend;
use binius_math::DefaultEvaluationDomainFactory;
use groestl_crypto::Groestl256;
use binius_hash::compress::Groestl256ByteCompression;

pub fn prove_verify_test(
    witness: MultilinearExtensionIndex<OptimalUnderlier, BinaryField128b>,
    constraints: ConstraintSystem<BinaryField128b>,
) -> (bool, bool) {

    let mut buf = vec![];
    constraints.write(&mut buf).expect("constraints serialization issue");
    let constraints = ConstraintSystem::<BinaryField128b>::read(buf.as_slice()).expect("constraints deserialization issue");
    println!("serialization is OK");

    let domain_factory = DefaultEvaluationDomainFactory::default();
    let backend = make_portable_backend();
    let proof = constraint_system::prove::<
        OptimalUnderlier,
        CanonicalTowerFamily,
        BinaryField8b,
        _,
        Groestl256,
        Groestl256ByteCompression,
        HasherChallenger<Groestl256>,
        _,
    >(&constraints, 1usize, 100usize, witness, &domain_factory, &backend);

    let prove_no_issues = proof.is_ok();
    if !prove_no_issues {
        // Since we have issue on proving, verification is also an issue
        return (false, false);
    }

    let out = constraint_system::verify::<
        OptimalUnderlier,
        CanonicalTowerFamily,
        Groestl256,
        Groestl256ByteCompression,
        HasherChallenger<Groestl256>,
    >(&constraints, 1usize, 100usize, vec![], proof.unwrap());

    let verify_no_issues = out.is_ok();
    (prove_no_issues, verify_no_issues)
}
