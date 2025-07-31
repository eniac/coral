// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use toml_core::Outputs;
use toml_methods::{VALIDATE_TOML_ELF, VALIDATE_TOML_ID};
use std::process;
use std::fs;
use std::env;
use risc0_zkvm::{default_prover, ExecutorEnv};
use csv::Writer;
use std::time::SystemTime;
use std::fs::OpenOptions;
use std::path::PathBuf;


#[cfg(feature = "metrics")]
use metrics::metrics::{log, log::Component};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        process::exit(1);
    }

    let data_file = &args[1];

    let data =  match fs::read_to_string(data_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading input file: {}", e);
            process::exit(1);
        }
    };
    #[cfg(feature = "metrics")]{
        assert!(args.len() == 3, "Expected metrics file");
        let metrics = PathBuf::from(args[2].clone());
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&metrics)
            .unwrap();
        let mut wtr = Writer::from_writer(file);
        let _ = wtr.write_record(&[
            data_file.to_string(),
            "toml_risc0".to_string(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
            data.len().to_string(),
        ]);
        let spacer = "---------";
        let _ = wtr.write_record(&[spacer, spacer, spacer, spacer, "\n"]);
        let _ = wtr.flush();
        #[cfg(feature = "metrics")]
        log::write_csv(metrics.to_str().unwrap()).unwrap();
    }

    let outputs = validate_toml(&data);
    println!("{:?} is a valid toml - {:?}", outputs.hash, outputs.valid);

    #[cfg(feature = "metrics")]
    log::write_csv(&PathBuf::from(args[2].clone()).as_path().display().to_string()).unwrap();
}

fn validate_toml(data: &str) -> Outputs {

    #[cfg(feature = "metrics")]
    log::tic(Component::Compiler, "compilation");
    let env = ExecutorEnv::builder()
        .write(&data)
        .unwrap()
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    #[cfg(feature = "metrics")] {
        log::stop(Component::Compiler, "compilation");
        log::tic(Component::Prover, "prove");
    }

    // Produce a receipt by proving the specified ELF binary.
    let receipt = prover.prove(env, VALIDATE_TOML_ELF).unwrap().receipt;

    #[cfg(feature = "metrics")]
    {
        log::stop(Component::Prover, "prove");
        log::space(Component::Prover, "receipt", receipt.seal_size());
        log::tic(Component::Verifier, "verify");    
    }

    assert!(receipt.verify(VALIDATE_TOML_ID).is_ok());

    #[cfg(feature = "metrics")]
    log::stop(Component::Verifier, "verify");

    receipt.journal.decode().unwrap()
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn main() {
//         let data = include_str!("../res/example.json");
//         let outputs = super::search_json(data);
//         assert_eq!(
//             outputs.data, 47,
//             "Did not find the expected value in the critical_data field"
//         );
//     }
// }
