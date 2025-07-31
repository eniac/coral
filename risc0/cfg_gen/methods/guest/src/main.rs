#![no_main]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use risc0_zkvm::guest::env;
use pest_vm::Vm;
use risc0_zkvm::sha::{Impl, Sha256};
use cfg_core::Outputs;
use pest_meta::optimizer::OptimizedRule;

risc0_zkvm::guest::entry!(main);

const ENTRY_RULE: &str = "root";

fn main() {
    env::log("Guest program starting");

    // Load the grammar string from the host
    let grammar_str: String = env::read();
    env::log(&format!("Grammar loaded, length: {}", grammar_str.len()));

    // Compile the grammar within the guest
    env::log("Compiling grammar...");
    let compiled_grammar = match compile_grammar(&grammar_str) {
        Ok(grammar) => {
            env::log("Grammar compiled successfully");
            grammar
        },
        Err(e) => {
            env::log(&format!("Grammar compilation failed: {}", e));
            env::commit(&false);
            return; // Add return here to ensure we exit
        }
    };

    // Load the input data string from the host
    let input_data: String = env::read();
    env::log(&format!("Input data loaded, length: {}", input_data.len()));

    // Create VM from compiled grammar
    env::log("Creating VM from compiled grammar");
    let vm = Vm::new(compiled_grammar);
    env::log("VM created successfully");

    // Try to parse the input data using the compiled grammar
    env::log(&format!("Attempting to parse input with rule: {}", ENTRY_RULE));
    let parse_result = vm.parse(ENTRY_RULE, &input_data);

    let hash = *Impl::hash_bytes(&input_data.as_bytes());

    let out = Outputs { 
        valid: parse_result.is_ok(),
        hash,
    };
    env::commit(&out);
    env::log("Committed result, exiting");
}

// Function to compile a grammar string into Vec<OptimizedRule>
fn compile_grammar(grammar_str: &str) -> Result<Vec<OptimizedRule>, String> {
    use pest_meta::parser::{self, Rule};
    use pest_meta::optimizer;

    // Parse the grammar
    let pairs = match parser::parse(Rule::grammar_rules, grammar_str) {
        Ok(pairs) => pairs,
        Err(e) => return Err(format!("Failed to parse grammar: {}", e)),
    };

    // Process the AST
    let ast = match parser::consume_rules(pairs) {
        Ok(ast) => ast,
        Err(errors) => {
            let err_msg = errors.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!("Failed to process grammar AST: {}", err_msg));
        }
    };

    // Optimize the rules
    let optimized = optimizer::optimize(ast);
    
    Ok(optimized)
}