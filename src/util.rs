use crate::{parser::GrammarGraph, prover::make_coral_circuit, solver::*};
use ark_bn254::Bn254;
use ark_ff::{BigInteger256, FftField, PrimeField};
use ark_poly::univariate::DensePolynomial;
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::kzg10::{self, Powers, UniversalParams, VerifierKey};
use ark_poly_commit::Error;
use csv::Writer;
use nova_snark::{
    nova::PublicParams,
    provider::{Bn256EngineKZG, GrumpkinEngine},
    traits::{snark::default_ck_hint, Engine},
};
use rand::rngs::OsRng;
use segmented_circuit_memory::bellpepper::FCircuit;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::time::SystemTime;
use std::{fs, usize};

#[cfg(feature = "metrics")]
use metrics::metrics::{log, log::Component};

pub trait ArkPrimeField = PrimeField<BigInt = BigInteger256>;

pub type AF = ark_bn254::Fr;
pub type PolyBn254 = DensePolynomial<AF>;
pub type ArkKZG = kzg10::KZG10<Bn254, PolyBn254>;

pub type HashMap<K, V> = std::collections::HashMap<K, V>;
pub type HashSet<K> = std::collections::HashSet<K>;

pub fn new_hash_map<K, V>() -> HashMap<K, V> {
    HashMap::default()
}

pub type LDE<F> = DensePolynomial<F>;

pub type E1 = Bn256EngineKZG;
pub type E2 = GrumpkinEngine;
pub type EE1 = nova_snark::provider::hyperkzg::EvaluationEngine<E1>;
pub type EE2 = nova_snark::provider::ipa_pc::EvaluationEngine<E2>;
pub type S1 = nova_snark::spartan::snark::RelaxedR1CSSNARK<E1, EE1>;
pub type S2 = nova_snark::spartan::snark::RelaxedR1CSSNARK<E2, EE2>;
pub type C1 = FCircuit<<E1 as Engine>::Scalar>;
pub type N1 = <E1 as Engine>::Scalar;
pub type N2 = <E2 as Engine>::Scalar;

pub fn get_name(opt_1: Option<String>, cmt_or_prf: bool, p_or_v: bool) -> String {
    if opt_1.is_some() {
        if p_or_v {
            format!("p_{}", opt_1.unwrap())
        } else {
            format!("v_{}", opt_1.unwrap())
        }
    } else {
        if cmt_or_prf {
            if p_or_v {
                return "prover_data.cmt".to_string();
            } else {
                return "verifier_data.cmt".to_string();
            }
        } else {
            return "to_verify.proof".to_string();
        }
    }
}

pub fn build_root_products<F: FftField>(roots: &[F]) -> LDE<F> {
    let l = roots.len();

    if l == 1 {
        return LDE::from_coefficients_vec(vec![-roots[0], F::one()]);
    }

    let mid = l / 2;
    let (left, right) = roots.split_at(mid);

    let left_poly = build_root_products(left);
    let right_poly = build_root_products(right);

    left_poly * right_poly
}

pub fn build_root_products_para<F: FftField>(roots: &[F]) -> LDE<F> {
    let l = roots.len();

    if l == 1 {
        return LDE::from_coefficients_vec(vec![-roots[0], F::one()]);
    }

    let mid = l / 2;
    let (left, right) = roots.split_at(mid);

    // Parallelize the recursive calls using Rayon
    let (left_poly, right_poly) = rayon::join(
        || build_root_products(left),  // Run in parallel
        || build_root_products(right), // Run in parallel
    );

    left_poly * right_poly
}

pub fn read_graph(pest_file: String, input: String) -> (GrammarGraph, Vec<char>) {
    let grammar = fs::read_to_string(pest_file).expect("Failed to read grammar file");
    let input_text = fs::read_to_string(input).expect("Failed to read input file");

    let mut grammar_graph = GrammarGraph::new();
    grammar_graph
        .parse_text_and_build_graph(&grammar, &input_text)
        .expect("Failed to parse input");

    grammar_graph.parse_and_convert_lcrs();
    (grammar_graph, input_text.chars().collect())
}

pub fn gen_pp<AF: ArkPrimeField>(empty_csc: &mut CoralStepCircuit<AF>) -> PublicParams<E1, E2, C1> {
    #[cfg(feature = "metrics")]
    log::tic(Component::Generator, "nova_pp_gen");
    let mut irw = InterRoundWires::new();

    let mut circuit_primary = make_coral_circuit(empty_csc, &mut irw, 0, None);

    let pp = PublicParams::<E1, E2, C1>::setup(
        &mut circuit_primary,
        &*default_ck_hint(),
        &*default_ck_hint(),
        vec![empty_csc.key_length],
        Some("./ppot_0080_23.ptau"),
    )
    .unwrap();
    #[cfg(feature = "metrics")]
    log::stop(Component::Generator, "nova_pp_gen");
    pp
}

pub fn trim<'a>(
    pp: UniversalParams<Bn254>,
    mut supported_degree: usize,
) -> Result<(Powers<'a, Bn254>, VerifierKey<Bn254>), Error> {
    if supported_degree == 1 {
        supported_degree += 1;
    }
    let powers_of_g = pp.powers_of_g[..=supported_degree].to_vec();
    let powers_of_gamma_g = (0..=supported_degree)
        .map(|i| pp.powers_of_gamma_g[&i])
        .collect();

    let powers = Powers {
        powers_of_g: ark_std::borrow::Cow::Owned(powers_of_g),
        powers_of_gamma_g: ark_std::borrow::Cow::Owned(powers_of_gamma_g),
    };
    let vk = VerifierKey {
        g: pp.powers_of_g[0],
        gamma_g: pp.powers_of_gamma_g[&0],
        h: pp.h,
        beta_h: pp.beta_h,
        prepared_h: pp.prepared_h.clone(),
        prepared_beta_h: pp.prepared_beta_h.clone(),
    };
    Ok((powers, vk))
}

pub fn gen_ark_pp<'a>(doc_len: usize) -> (Powers<'a, Bn254>, kzg10::VerifierKey<Bn254>) {
    let rng = &mut OsRng;
    let ark_kzg_pp = ArkKZG::setup(doc_len + 1, false, rng).unwrap();

    let (ck, vk) = trim(ark_kzg_pp, doc_len + 1).unwrap();

    (ck, vk)
}

pub fn metrics_file(
    metrics: Option<PathBuf>,
    grammar: &String,
    doc: &String,
    doc_len: usize,
    tree_size: usize,
    batch_size: usize,
    grammar_len: usize,
) {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(metrics.clone().unwrap())
        .unwrap();
    let mut wtr = Writer::from_writer(file);
    let _ = wtr.write_record(&[
        doc.to_string(),
        grammar.to_string(),
        time,
        grammar_len.to_string(),
        tree_size.to_string(),
        doc_len.to_string(),
        batch_size.to_string(),
    ]);
    let spacer = "---------";
    let _ = wtr.write_record([spacer, spacer, spacer, spacer, "\n"]);
    let _ = wtr.flush();
    #[cfg(feature = "metrics")]
    log::write_csv(metrics.unwrap().to_str().unwrap()).unwrap();
}
