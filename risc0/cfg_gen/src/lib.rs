#![doc = include_str!("../README.md")]

use cfg_core::Outputs;
use cfg_validation::{VALIDATE_INPUT_ELF, VALIDATE_INPUT_ID};
use csv::Writer;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process;
use std::time::SystemTime;

#[cfg(feature = "metrics")]
use metrics::metrics::{log, log::Component};

// Updated validate_input function - much simpler approach
pub fn validate_input(grammar_content: &str, input_data: &str) -> (Receipt, Outputs) {
    println!("Starting validate_input function");
    println!(
        "Grammar length: {}, Input length: {}",
        grammar_content.len(),
        input_data.len()
    );

    println!("Preparing to send grammar string to zkVM...");

    #[cfg(feature = "metrics")]
    log::tic(Component::Compiler, "compilation");

    let env = ExecutorEnv::builder()
        .write(&grammar_content)
        .unwrap()
        .write(&input_data)
        .unwrap()
        .build()
        .unwrap();

    println!("Environment built successfully");

    // Obtain the default prover.
    let prover = default_prover();
    println!("Default prover obtained");

    #[cfg(feature = "metrics")]
    {
        log::stop(Component::Compiler, "compilation");
        log::tic(Component::Prover, "prove");
    }

    println!("Starting proof generation - this might take time...");
    let receipt = prover.prove(env, VALIDATE_INPUT_ELF).unwrap().receipt;
    println!("Proof generation completed");

    #[cfg(feature = "metrics")]
    {
        log::stop(Component::Prover, "prove");
        log::space(Component::Prover, "receipt", receipt.seal_size());
        log::tic(Component::Verifier, "verify");
    }

    let is_valid: Outputs = receipt
        .journal
        .decode()
        .expect("Journal output should deserialize");

    assert!(receipt.verify(VALIDATE_INPUT_ID).is_ok());

    #[cfg(feature = "metrics")]
    log::stop(Component::Verifier, "verify");

    // Report the result
    println!(
        "ZKVM guest determined input {:?} is valid - {} according to the provided grammar.",
        is_valid.hash, is_valid.valid
    );
    (receipt, is_valid)
}
