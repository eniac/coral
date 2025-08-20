#![no_main]
sp1_zkvm::entrypoint!(main);

use lib::{Account, Transaction}; // Custom structs.
use serde_json::Value; // Generic JSON.

pub fn main() {
    // read generic JSON example inputs.
    let data_str = sp1_zkvm::io::read::<String>();
    let v: Value = serde_json::from_str(&data_str).unwrap();
    let ok = "1";
    sp1_zkvm::io::commit(&ok);
}
