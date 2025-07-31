#![allow(missing_docs, non_snake_case)]
#![feature(box_patterns)]
mod parser;
use anyhow::Result;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use clap::Parser;
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
// use petgraph::dot::{Config, Dot};
// use std::fs::File;
// use std::io::Write;
fn main() -> Result<()> {
    let opt = Options::parse();

    let grammar_path = opt.grammar;
    let input_text_path = opt.doc;
    let batch_size = opt.batch_size;

    let (grammar_graph, doc) = read_graph(grammar_path.clone(), input_text_path.clone());

    if opt.e2e || opt.commit {
        #[cfg(feature = "metrics")]
        log::tic(Component::Generator, "doc_commit_params");

        let (ark_ck, ark_vk) = gen_ark_pp(doc.len());

        #[cfg(feature = "metrics")]
        log::stop(Component::Generator, "doc_commit_params");

        let doc_commit = run_doc_committer(doc.clone(), &ark_ck);

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
        let (mut p_i, mut base, mut empty, pp) =
            prover::setup(&grammar_graph, batch_size, prover_doc_commit.blind).unwrap();

        #[cfg(feature = "para")]
        let prover_output_res =
            run_para_prover::<AF>(&grammar_graph, base, &mut p_i, prover_doc_commit, &pp);

        #[cfg(not(feature = "para"))]
        let prover_output_res =
            run_prover::<AF>(&grammar_graph, &mut base, &mut p_i, prover_doc_commit, &pp);

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
        metrics_file(
            opt.metrics.clone(),
            &grammar_path,
            &input_text_path,
            doc.len(),
            grammar_graph.lcrs_tree.node_count(),
            opt.batch_size,
            grammar_graph.rule_count,
        );
        log::write_csv(&opt.metrics.clone().unwrap().as_path().display().to_string()).unwrap();
    }
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use coral::verifier;
//     use coral::{
//         prover::{self, *},
//         util::*,
//         verifier::*,
//     };
//     use nova_snark::nova::CompressedSNARK;

//     #[test]
//     pub fn test_e2e() {
//         let (grammar_graph, doc) = read_graph(
//             "./grammars/test_any.pest".to_string(),
//             "./tests/test_docs/test_any.txt".to_string(),
//         );

//         let batch_size = 2;

//         let (ark_ck, ark_vk) = gen_ark_pp(doc.len());

//         let mut p_i = run_doc_committer(doc, &ark_ck);

//         let (mut base, mut empty, pp) =
//             prover::setup(&grammar_graph, &mut p_i, batch_size).unwrap();

//         let mut v_i = verifier::setup(
//             &mut empty,
//             p_i.doc_commit,
//             ark_vk,
//             p_i.ic_cmt.clone().unwrap(),
//         );

//         // produce the prover and verifier keys for compressed snark
//         let (pk, vk) = CompressedSNARK::<_, _, _, S1, S2>::setup(&pp).unwrap();

//         p_i.snark_pk = Some(pk);
//         v_i.snark_vk = Some(vk);

//         let prover_output = run_prover::<AF>(&grammar_graph, &mut base, &mut p_i, &pp);

//         assert!(prover_output.is_ok());

//         let verifier_output = verify(&mut prover_output.unwrap(), v_i);

//         assert!(verifier_output.is_ok());
//     }
// }
