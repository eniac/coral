use crate::{prover::ProverOutput, solver::*, util::*};
use ark_bn254::Bn254;
use ark_ff::PrimeField as arkPrimeField;
use ark_poly_commit::kzg10;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use nova_snark::nova::CompressedSNARK;
use nova_snark::{
    errors::NovaError,
    nova::{PublicParams, VerifierKey},
};
use segmented_circuit_memory::memory::nebula::RunningMem;

use std::usize;

#[cfg(feature = "metrics")]
use metrics::metrics::{log, log::Component};

pub struct VerifierInfo<ArkF: arkPrimeField> {
    pub tree_size: usize,
    pub pp: PublicParams<E1, E2, C1>,
    pub num_steps: usize,
    pub mem: RunningMem<AF>,
    pub snark_vk: VerifierKey<E1, E2, C1, S1, S2>,
    pub perm_chal: Vec<ArkF>,
}

#[derive(CanonicalSerialize, CanonicalDeserialize, Debug)]
pub struct VerifierDocCommit {
    pub doc_commit: kzg10::Commitment<Bn254>,
    pub doc_commit_vk: kzg10::VerifierKey<Bn254>,
}

pub fn setup(empty_circuit: &mut CoralStepCircuit<AF>) -> VerifierInfo<AF> {
    let pp = gen_pp(empty_circuit);

    #[cfg(feature = "metrics")]
    log::tic(Component::Verifier, "snark_params");
    let (_, vk) = CompressedSNARK::<_, _, _, S1, S2>::setup(&pp).unwrap();

    #[cfg(feature = "metrics")]
    log::stop(Component::Verifier, "snark_params");

    VerifierInfo {
        tree_size: empty_circuit.tree_size_usize,
        pp,
        num_steps: usize::div_ceil(empty_circuit.tree_size_usize, empty_circuit.batch_size),
        mem: empty_circuit.mem.clone().unwrap(),
        snark_vk: vk,
        perm_chal: empty_circuit.mem.as_ref().unwrap().perm_chal.clone(),
    }
}

pub fn verify(
    p_o: &mut ProverOutput,
    v_i: VerifierInfo<AF>,
    v_dc: VerifierDocCommit,
) -> Result<(), NovaError> {
    #[cfg(feature = "metrics")]
    log::tic(Component::Verifier, "full_verify");

    #[cfg(feature = "metrics")]
    log::tic(Component::Verifier, "snark_verify");

    let comp_snark_result = p_o
        .compressed_snark
        .verify(&v_i.snark_vk, v_i.num_steps, &p_o.z_0);
    assert!(comp_snark_result.is_ok());

    #[cfg(feature = "metrics")]
    log::stop(Component::Verifier, "snark_verify");

    // check final cmt outputs
    let (zn, ci) = comp_snark_result.unwrap();

    #[cfg(feature = "metrics")]
    log::tic(Component::Verifier, "eq_checks");

    v_i.mem.verifier_checks(&zn, &ci, false);

    //Have to get stack ptrs out
    let sp_offset = 11;
    let sp_0 = zn[sp_offset];
    let sp_1 = zn[sp_offset + 1];

    assert_eq!(sp_0, N1::from(1));
    assert_eq!(sp_1, N1::from(1));

    #[cfg(feature = "metrics")]
    {
        log::stop(Component::Verifier, "eq_checks");
        log::tic(Component::Verifier, "doc_commit_check");
    }

    //Have to get calimed eval out of zn
    //Check doc commitment
    let eval_offset = sp_offset + 2;
    let claimed_eval = zn[eval_offset];
    let kzg_check = ArkKZG::check(
        &v_dc.doc_commit_vk,
        &v_dc.doc_commit,
        v_i.perm_chal[0],
        segmented_circuit_memory::bellpepper::nova_to_ark_field(&claimed_eval),
        p_o.doc_commit_proof.as_ref().unwrap(),
    )
    .unwrap();
    assert!(kzg_check);

    println!("Verified Successfully!");

    #[cfg(feature = "metrics")]
    log::tic(Component::Verifier, "doc_commit_check");

    #[cfg(feature = "metrics")]
    log::stop(Component::Verifier, "full_verify");

    Ok(())
}
