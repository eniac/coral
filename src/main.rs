#![allow(missing_docs, non_snake_case)]

mod parser;
use anyhow::Result;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use clap::Parser;
use coral::parser::GrammarGraph;
use coral::verifier::{self, VerifierDocCommit};
use coral::{
    config::*,
    prover::{self, *},
    util::*,
    verifier::verify,
};
use std::fs;

#[cfg(feature = "metrics")]
use metrics::metrics::{log, log::Component};
fn main() -> Result<()> {
    let opt = Options::parse();

    let grammar_path = opt.grammar;
    let input_text_path = opt.doc;
    let batch_size = opt.batch_size;

    let mut opt_grammar_graph: Option<GrammarGraph> = None;
    let mut opt_doc: Option<Vec<char>> = None;

    if opt.commit || opt.prove || opt.e2e {
        assert!(
            input_text_path.is_some(),
            "Input text file must be provided for commit or prove"
        );

        let (grammar_graph, doc) = read_graph(
            grammar_path.clone(),
            input_text_path.as_ref().unwrap().clone(),
        );

        opt_grammar_graph = Some(grammar_graph);
        opt_doc = Some(doc);
    }

    if opt.e2e || opt.commit {
        #[cfg(feature = "metrics")]
        log::tic(Component::Generator, "doc_commit_params");

        let (ark_ck, ark_vk) = gen_ark_pp(opt_doc.as_ref().unwrap().len());

        #[cfg(feature = "metrics")]
        log::stop(Component::Generator, "doc_commit_params");

        let doc_commit = run_doc_committer(opt_doc.as_ref().unwrap(), &ark_ck);

        let mut prover_compressed_bytes = Vec::new();
        doc_commit
            .serialize_compressed(&mut prover_compressed_bytes)
            .unwrap();
        fs::write(
            get_name(opt.cmt_name.clone(), true, true),
            prover_compressed_bytes,
        )
        .expect("Unable to write file");

        let v_doc_commit = verifier::VerifierDocCommit {
            doc_commit: doc_commit.doc_commit,
            doc_commit_vk: ark_vk,
        };
        let mut verifier_compressed_bytes = Vec::new();
        v_doc_commit
            .serialize_compressed(&mut verifier_compressed_bytes)
            .unwrap();
        fs::write(
            get_name(opt.cmt_name.clone(), true, false),
            verifier_compressed_bytes,
        )
        .expect("Unable to write file");
    }

    if opt.e2e || opt.prove {
        // read commitment
        let prover_cmt_data =
            fs::read(get_name(opt.cmt_name.clone(), true, true)).expect("Unable to read file");

        let prover_doc_commit: CoralDocCommitment =
            CoralDocCommitment::deserialize_compressed_unchecked(&*prover_cmt_data).unwrap();

        #[allow(unused_mut)]
        let (mut p_i, mut base, mut empty, pp) = prover::setup(
            opt_grammar_graph.as_ref().unwrap(),
            batch_size,
            prover_doc_commit.blind,
        )
        .unwrap();

        #[cfg(feature = "para")]
        let prover_output_res = run_para_prover::<AF>(
            opt_grammar_graph.as_ref().unwrap(),
            base,
            &mut p_i,
            prover_doc_commit,
            &pp,
        );

        #[cfg(not(feature = "para"))]
        let prover_output_res = run_prover::<AF>(
            opt_grammar_graph.as_ref().unwrap(),
            &mut base,
            &mut p_i,
            prover_doc_commit,
            &pp,
        );

        assert!(prover_output_res.is_ok());

        let mut prover_output = prover_output_res.unwrap();
        prover_output.empty = Some(empty);

        let prover_output_data = bincode::serialize(&prover_output).unwrap();
        fs::write(
            get_name(opt.proof_name.clone(), false, false),
            prover_output_data,
        )
        .expect("Unable to write file");
    }
    if opt.e2e || opt.verify {
        let data_from_prover =
            fs::read(get_name(opt.proof_name.clone(), false, false)).expect("Unable to read file");
        let mut prover_output = bincode::deserialize::<ProverOutput>(&data_from_prover).unwrap();

        let mut empty = prover_output.empty.take().unwrap();

        let v_i = verifier::setup(&mut empty);

        let verifer_doc_commit_data =
            fs::read(get_name(opt.cmt_name.clone(), true, false)).expect("Unable to read file");
        let verifier_doc_commit: VerifierDocCommit =
            VerifierDocCommit::deserialize_compressed_unchecked(&*verifer_doc_commit_data).unwrap();

        let verifier_output = verify(&mut prover_output, v_i, verifier_doc_commit);

        assert!(verifier_output.is_ok());
    }

    #[cfg(feature = "metrics")]
    {
        if opt_grammar_graph.is_some() && input_text_path.is_some() {
            metrics_file(
                opt.metrics.clone(),
                &grammar_path,
                &input_text_path.unwrap(),
                opt_doc.unwrap().len(),
                opt_grammar_graph.as_ref().unwrap().lcrs_tree.node_count(),
                opt.batch_size,
                opt_grammar_graph.as_ref().unwrap().rule_count,
            );
            log::write_csv(&opt.metrics.clone().unwrap().as_path().display().to_string()).unwrap();
        }
    }
    Ok(())
}
