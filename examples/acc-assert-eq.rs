use groestl_crypto::Groestl256;
use binius_circuits::builder::ConstraintSystemBuilder;
use binius_circuits::unconstrained::variable_u128;
use binius_core::constraint_system;
use binius_core::constraint_system::ConstraintSystem;
use binius_core::fiat_shamir::HasherChallenger;
use binius_core::tower::CanonicalTowerFamily;
use binius_core::witness::MultilinearExtensionIndex;
use binius_field::arch::OptimalUnderlier;
use binius_field::{BinaryField128b, BinaryField1b, BinaryField8b, Field, TowerField};
use binius_hal::make_portable_backend;
use binius_hash::{GroestlDigestCompression, GroestlHasher};
use binius_macros::arith_expr;
use binius_math::DefaultEvaluationDomainFactory;

const LOG_SIZE: usize = 10;
const ARRAY_SIZE: usize = 10000;

fn assert_eq_gadget(builder: &mut ConstraintSystemBuilder<OptimalUnderlier, BinaryField128b>, input: &[u128]) {
    let bit_sum_value = input.into_iter().sum::<u128>();
    let bit_sum_id = variable_u128::<_, _, BinaryField1b>(builder, "bitsum", LOG_SIZE, bit_sum_value).unwrap();
    let zero_id = variable_u128::<_, _, BinaryField1b>(builder, "zero", LOG_SIZE, 0u128).unwrap();

    builder.assert_zero(
        [bit_sum_id, zero_id],
        arith_expr!([bit, zero] = bit - zero).convert_field(),
    );
}

fn main() {
    // Positive test (input is all zeroes)
    let mut input = [0u128; ARRAY_SIZE];

    let allocator = bumpalo::Bump::new();
    let mut builder = ConstraintSystemBuilder::<OptimalUnderlier, BinaryField128b>::new_with_witness(&allocator);

    assert_eq_gadget(&mut builder, input.as_slice());

    let witness = builder.take_witness().unwrap();
    let cs = builder.build().unwrap();

    let (prove_no_issues, verify_no_issues) = prove_verify_test(witness, cs);
    assert!(prove_no_issues);
    assert!(verify_no_issues);

    // Negative test (input contains non-zero element)
    input[ARRAY_SIZE - 1] = 1u128;
    let allocator = bumpalo::Bump::new();
    let mut builder = ConstraintSystemBuilder::<OptimalUnderlier, BinaryField128b>::new_with_witness(&allocator);

    assert_eq_gadget(&mut builder, input.as_slice());

    let witness = builder.take_witness().unwrap();
    let cs = builder.build().unwrap();

    let (prove_no_issues, verify_no_issues) = prove_verify_test(witness, cs);
    assert!(prove_no_issues);
    assert!(!verify_no_issues); // issue on verification
}

fn prove_verify_test(
    witness: MultilinearExtensionIndex<OptimalUnderlier, BinaryField128b>,
    constraints: ConstraintSystem<BinaryField128b>,
) -> (bool, bool) {
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
    >(
        &constraints,
        1usize,
        100usize,
        witness,
        &domain_factory,
        &backend
    );

    let prove_no_issues = proof.is_ok();
    if !prove_no_issues {
        // Since we have issue on proving, verification is also an issue
        return (false, false)
    }

    let out = constraint_system::verify::<
        OptimalUnderlier,
        CanonicalTowerFamily,
        _,
        _,
        GroestlHasher<BinaryField128b>,
        GroestlDigestCompression<BinaryField8b>,
        HasherChallenger<Groestl256>,
    >(
        &constraints,
        1usize,
        100usize,
        &domain_factory,
        vec![],
        proof.unwrap()
    );

    let verify_no_issues = out.is_ok();
    (prove_no_issues, verify_no_issues)
}