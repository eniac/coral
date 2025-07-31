use crate::{circuit::multi_node_step, parser::GrammarGraph, solver::*, util::*};
use ark_bn254::Bn254;
use ark_poly_commit::kzg10::{self, Commitment, Powers};
use ark_relations::gr1cs::{ConstraintSystem, OptimizationGoal, SynthesisError, SynthesisMode};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

use ark_serialize::CompressedChecked;
use ark_std::UniformRand;
use nova_snark::{
    errors::NovaError,
    frontend::LinearCombination,
    nova::{CompressedSNARK, ProverKey, PublicParams, RandomLayer, RecursiveSNARK},
};
use rand::rngs::OsRng;
use segmented_circuit_memory::bellpepper::FCircuit;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::usize;
use std::{
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};

#[cfg(feature = "metrics")]
use metrics::metrics::{log, log::Component};

pub struct ProverInfo {
    pub ic_key_length: usize,
    pub ic_blinds: Vec<Vec<N1>>,
    pub ic_hints: Vec<Vec<N1>>,
    pub snark_pk: ProverKey<E1, E2, C1, S1, S2>,
    pub random_layer: RandomLayer<E1, E2>,
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize)]
pub struct ProverOutput {
    pub compressed_snark: CompressedSNARK<E1, E2, C1, S1, S2>,
    #[serde_as(as = "CompressedChecked<Option<CoralStepCircuit<AF>>>")]
    pub empty: Option<CoralStepCircuit<AF>>,
    #[serde_as(as = "CompressedChecked<Option<kzg10::Proof<Bn254>>>")]
    pub doc_commit_proof: Option<kzg10::Proof<Bn254>>,
    pub z_0: Vec<N1>,
}
#[derive(CanonicalSerialize, CanonicalDeserialize)]

pub struct CoralDocCommitment<'b> {
    pub blind: AF,
    pub doc_commit: Commitment<Bn254>,
    pub commit_rand: kzg10::Randomness<AF, PolyBn254>,
    pub doc_commit_poly: PolyBn254,
    pub doc_ck: Powers<'b, Bn254>,
}

pub fn run_doc_committer<'a>(doc: Vec<char>, ck: &Powers<'a, Bn254>) -> CoralDocCommitment<'a> {
    #[cfg(feature = "metrics")]
    log::tic(Component::Generator, "doc_commit");

    let blind = AF::rand(&mut OsRng);

    let shift = AF::from(2_u64.pow(32));

    let mut doc_roots = doc
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let base_shift = shift * (coral_hash::<AF>(&c.to_string()));
            base_shift + to_F::<AF>(i)
        })
        .collect::<Vec<_>>();
    doc_roots.push(blind);

    #[cfg(feature = "para")]
    let doc_commit_poly = build_root_products_para::<AF>(&doc_roots[..]);

    #[cfg(not(feature = "para"))]
    let doc_commit_poly = build_root_products::<AF>(&doc_roots[..]);

    let (comms, rand) = ArkKZG::commit(ck, &doc_commit_poly, Some(1), Some(&mut OsRng)).unwrap();

    #[cfg(feature = "metrics")]
    log::stop(Component::Generator, "doc_commit");

    CoralDocCommitment {
        blind,
        doc_commit: comms,
        commit_rand: rand,
        doc_commit_poly,
        doc_ck: ck.clone(),
    }
}

pub fn setup<ArkF: ArkPrimeField>(
    grammar_graph: &GrammarGraph,
    batch_size: usize,
    doc_blind: ArkF,
) -> Result<
    (
        ProverInfo,
        CoralStepCircuit<ArkF>,
        CoralStepCircuit<ArkF>,
        PublicParams<E1, E2, C1>,
    ),
    SynthesisError,
> {
    let mut base = CoralStepCircuit::new(grammar_graph, batch_size, doc_blind);

    let (ic_blinds, ram_hints, mut empty) = base.solve(grammar_graph)?;

    let pp = gen_pp(&mut empty);

    #[cfg(feature = "metrics")]
    log::tic(Component::Prover, "sample_random_layer");

    let random_layer = CompressedSNARK::<_, _, _, S1, S2>::sample_random_layer(&pp).unwrap();

    #[cfg(feature = "metrics")]
    log::stop(Component::Prover, "sample_random_layer");

    #[cfg(feature = "metrics")]
    log::tic(Component::Prover, "snark_params");
    let (pk, _) = CompressedSNARK::<_, _, _, S1, S2>::setup(&pp).unwrap();

    #[cfg(feature = "metrics")]
    log::stop(Component::Prover, "snark_params");

    let p_i = ProverInfo {
        ic_key_length: base.key_length,
        ic_blinds: ic_blinds,
        ic_hints: ram_hints,
        snark_pk: pk,
        random_layer: random_layer,
    };

    Ok((p_i, base, empty, pp))
}

type Constraint<F> = (
    LinearCombination<F>,
    LinearCombination<F>,
    LinearCombination<F>,
);

pub fn make_coral_circuit<ArkF: ArkPrimeField>(
    csc: &mut CoralStepCircuit<ArkF>,
    irw: &mut InterRoundWires<ArkF>,
    i: usize,
    saved_matrix: Option<Arc<Vec<Constraint<N1>>>>,
) -> FCircuit<N1> {
    #[cfg(feature = "metrics")]
    {
        log::tic(Component::Solver, format!("witness_synthesis_{}", i));
        log::tic(Component::Solver, format!("witness_synthesis_ark_{}", i));
    }

    let cs = ConstraintSystem::<ArkF>::new_ref();
    cs.set_optimization_goal(OptimizationGoal::Constraints);

    if i != 0 {
        cs.set_mode(SynthesisMode::Prove {
            construct_matrices: false,
            generate_lc_assignments: false,
        });
        let num_constraints = saved_matrix.as_ref().unwrap().len();
        cs.borrow_mut()
            .unwrap()
            .assignments
            .witness_assignment
            .reserve(num_constraints * 2);
    }

    let mut wires = CoralWires::wires_from_irw(irw, cs.clone(), csc, i);

    let mut memory = csc
        .mem
        .as_mut()
        .unwrap()
        .begin_new_circuit(cs.clone())
        .unwrap();

    let wires_res = multi_node_step(csc, &mut wires, &mut memory, cs.clone());

    assert!(wires_res.is_ok(), "Wires failed at {:?}", i);

    irw.update(wires_res.unwrap());

    #[cfg(feature = "metrics")]
    {
        log::stop(Component::Solver, format!("witness_synthesis_ark_{}", i));
    }

    let f = FCircuit::<N1>::new(cs.clone(), saved_matrix);

    #[cfg(feature = "metrics")]
    {
        log::stop(Component::Solver, format!("witness_synthesis_{}", i));
    }

    f
}

pub fn run_wit_synth<'a>(
    sender: Sender<Option<FCircuit<N1>>>,
    saved_nova_matrices: Arc<Vec<Constraint<N1>>>,
    base: CoralStepCircuit<AF>,
    irw: InterRoundWires<AF>,
    n_rounds: usize,
) {
    println!("Solving thread starting...");
    let mut base = base;
    let mut irw = irw;

    for i in 0..n_rounds {
        if i + 1 < n_rounds {
            let circuit_primary = make_coral_circuit(
                &mut base,
                &mut irw,
                i + 1,
                Some(saved_nova_matrices.clone()),
            );
            sender.send(Some(circuit_primary)).unwrap();
        }
    }
}

pub fn run_prove(
    recv: Receiver<Option<FCircuit<N1>>>,
    recursive_snark: &mut RecursiveSNARK<E1, E2, C1>,
    p_i: &mut ProverInfo,
    pp: &PublicParams<E1, E2, C1>,
    circuit_primary: FCircuit<N1>,
    z0_primary: Vec<N1>,
    n_rounds: usize,
) -> Result<ProverOutput, NovaError> {
    let mut circuit_primary = circuit_primary;

    #[cfg(feature = "metrics")]
    log::tic(Component::Prover, "folding_proof");

    for i in 0..n_rounds {
        println!("Proving round {:?}", i);
        #[cfg(feature = "metrics")]
        log::tic(Component::Prover, format!("prove_{i}"));

        let res = recursive_snark.prove_step(
            pp,
            &mut circuit_primary,
            Some(p_i.ic_blinds[i].clone()),
            p_i.ic_hints[i].clone(),
            vec![p_i.ic_key_length.clone()],
        );
        assert!(res.is_ok());

        #[cfg(feature = "metrics")]
        log::stop(Component::Prover, format!("prove_{i}"));

        if i + 1 < n_rounds {
            circuit_primary = recv.recv().unwrap().unwrap();
        }
    }

    // produce a compressed SNARK
    #[cfg(feature = "metrics")]
    {
        log::stop(Component::Prover, "folding_proof");
        log::tic(Component::Prover, "compressed_snark");
    }

    println!("Compressed");
    let compressed_snark = CompressedSNARK::<_, _, _, S1, S2>::prove(
        pp,
        &p_i.snark_pk,
        recursive_snark,
        p_i.random_layer.clone(),
    );
    assert!(compressed_snark.is_ok());

    #[cfg(feature = "metrics")]
    log::stop(Component::Prover, "compressed_snark");

    Ok(ProverOutput {
        compressed_snark: compressed_snark.unwrap(),
        z_0: z0_primary,
        doc_commit_proof: None,
        empty: None,
    })
}

pub fn run_para_prover<ArkF: ArkPrimeField>(
    grammar_graph: &GrammarGraph,
    base: CoralStepCircuit<AF>,
    p_i: &mut ProverInfo,
    doc_commit: CoralDocCommitment<'_>,
    pp: &PublicParams<E1, E2, C1>,
) -> Result<ProverOutput, NovaError> {
    let n_rounds = u32::div_ceil(
        grammar_graph.lcrs_tree.node_count() as u32,
        base.batch_size as u32,
    ) as usize;

    let mut base = base;

    let perm_chal = base.mem.as_ref().unwrap().perm_chal.clone();

    let mut irw = InterRoundWires::new();

    let (sender_main, recv_main) = mpsc::channel();

    #[cfg(feature = "metrics")]
    log::tic(Component::Prover, "constraint_gen");

    let mut circuit_primary = make_coral_circuit(&mut base, &mut irw, 0, None);

    let z0_primary_full = circuit_primary.get_zi();
    let z0_offset = p_i.ic_key_length;
    let z0_primary = z0_primary_full[z0_offset..].to_vec();

    // produce a recursive SNARK
    let mut recursive_snark = RecursiveSNARK::<E1, E2, C1>::new(
        pp,
        &mut circuit_primary,
        &z0_primary,
        Some(p_i.ic_blinds[0].clone()),
        p_i.ic_hints[0].clone(),
        vec![p_i.ic_key_length.clone()],
    )
    .unwrap();

    let saved_nova_matrices = circuit_primary.lcs.as_ref().right().unwrap().clone();

    #[cfg(feature = "metrics")]
    {
        log::stop(Component::Prover, "constraint_gen");
        log::r1cs(Component::Prover, "Num Constraints", pp.num_constraints().0);
        log::tic(Component::Prover, "prove_e2e");
    }

    let doc_ck = doc_commit.doc_ck.clone();
    let doc_commit_poly = doc_commit.doc_commit_poly.clone();
    let commit_rand = doc_commit.commit_rand.clone();

    let (doc_proof_sender, doc_proof_recv) = mpsc::channel();
    let now = Instant::now();

    let mut prover_output = thread::scope(|s| {
        s.spawn(move || {
            run_wit_synth(sender_main, saved_nova_matrices, base, irw, n_rounds);
        });
        s.spawn(move || {
            #[cfg(feature = "metrics")]
            log::tic(Component::Prover, "doc_commit_proof");
            let doc_proof =
                ArkKZG::open(&doc_ck, &doc_commit_poly, perm_chal[0], &commit_rand).unwrap();
            doc_proof_sender
                .send(doc_proof)
                .expect("Failed to send doc proof");
            #[cfg(feature = "metrics")]
            log::stop(Component::Prover, "doc_commit_proof");
        });
        let handle3 = s.spawn(move || {
            run_prove(
                recv_main,
                &mut recursive_snark,
                p_i,
                pp,
                circuit_primary,
                z0_primary,
                n_rounds,
            )
        });
        handle3.join().expect("Proving thread panicked")
    })
    .unwrap();

    println!("Proving time: {:?}", now.elapsed());

    let proof_ark_kzg = doc_proof_recv.recv().expect("Failed to receive doc proof");

    prover_output.doc_commit_proof = Some(proof_ark_kzg);

    #[cfg(feature = "metrics")]
    {
        log::stop(Component::Prover, "prove_e2e");
        log::space(
            Component::Prover,
            "compressed_snark",
            bincode::serialize(&prover_output.compressed_snark)
                .unwrap()
                .len(),
        );
    }

    Ok(prover_output)
}

pub fn run_prover<ArkF: ArkPrimeField>(
    grammar_graph: &GrammarGraph,
    base: &mut CoralStepCircuit<AF>,
    p_i: &mut ProverInfo,
    doc_commit: CoralDocCommitment<'_>,
    pp: &PublicParams<E1, E2, C1>,
) -> Result<ProverOutput, NovaError> {
    let n_rounds = u32::div_ceil(
        grammar_graph.lcrs_tree.node_count() as u32,
        base.batch_size as u32,
    ) as usize;

    println!("n rounds {:?}", n_rounds);

    //Actually prove things now
    let mut irw = InterRoundWires::new();

    let mut circuit_primary = make_coral_circuit(base, &mut irw, 0, None);

    #[cfg(feature = "metrics")]
    log::r1cs(Component::Prover, "Num Constraints", pp.num_constraints().0);

    let z0_primary_full = circuit_primary.get_zi().clone();
    let z0_offset = p_i.ic_key_length;
    let z0_primary = z0_primary_full[z0_offset..].to_vec();

    // produce a recursive SNARK
    let mut recursive_snark = RecursiveSNARK::<E1, E2, C1>::new(
        pp,
        &mut circuit_primary,
        &z0_primary,
        Some(p_i.ic_blinds[0].clone()),
        p_i.ic_hints[0].clone(),
        vec![p_i.ic_key_length.clone()],
    )
    .unwrap();

    let saved_nova_matrices = circuit_primary.lcs.as_ref().right().unwrap().clone();

    #[cfg(feature = "metrics")]
    log::tic(Component::Prover, "prove_e2e");

    for i in 0..n_rounds {
        println!("Proving round {:?}", i);
        #[cfg(feature = "metrics")]
        log::tic(Component::Prover, format!("prove_{}", i));

        let res = recursive_snark.prove_step(
            pp,
            &mut circuit_primary,
            Some(p_i.ic_blinds[i].clone()),
            p_i.ic_hints[i].clone(),
            vec![p_i.ic_key_length.clone()],
        );
        assert!(res.is_ok());

        #[cfg(feature = "metrics")]
        {
            log::stop(Component::Prover, format!("prove_{}", i));
        }

        if i + 1 < n_rounds {
            println!("gen round {:?}", i + 1);
            circuit_primary =
                make_coral_circuit(base, &mut irw, i + 1, Some(saved_nova_matrices.clone()));
        }
    }

    // produce a compressed SNARK
    #[cfg(feature = "metrics")]
    log::tic(Component::Prover, "compressed_snark");

    let compressed_snark = CompressedSNARK::<_, _, _, S1, S2>::prove(
        pp,
        &p_i.snark_pk,
        &recursive_snark,
        p_i.random_layer.clone(),
    );
    assert!(compressed_snark.is_ok());

    #[cfg(feature = "metrics")]
    {
        log::stop(Component::Prover, "compressed_snark");
        log::stop(Component::Prover, "prove_e2e");
    }

    let proof_ark_kzg = ArkKZG::open(
        &doc_commit.doc_ck,
        &doc_commit.doc_commit_poly,
        base.mem.as_mut().unwrap().perm_chal[0],
        &doc_commit.commit_rand,
    )
    .unwrap();

    Ok(ProverOutput {
        compressed_snark: compressed_snark.unwrap(),
        z_0: z0_primary,
        doc_commit_proof: Some(proof_ark_kzg),
        empty: None,
    })
}
