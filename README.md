
# Coral

This is an implementation of Coral, a system for generating zero-knowledge proofs that a committed document matches is consistent with a public Context Free Grammar
The details of Coral are described in our paper: [Coral: Fast Succinct Non-Interactive Zero-Knowledge CFG Proofs](https://eprint.iacr.org/2023/1886).

## Compile

```
cargo build --release
```

With metrics:
```
cargo build --release --features metrics
```

With pipeined proving and witness generation:
```
cargo build --release --features para
```


## Usage
```
Usage: coral [OPTIONS] <--commit|--prove|--verify|--e2e>

Options:
      --commit
      --prove
      --verify
      --e2e
      --cmt-name <FILE>     Optional name for .cmt file
      --proof-name <FILE>   Optional name for .proof file
  -d, --doc <FILE>
  -m, --metrics <FILE>      Metrics and other output information
  -g, --grammar <FILE>      .pest file containing the grammar
  -b, --batch-size <USIZE>  Batch size [default: 1]
  -h, --help                Print help
  -V, --version             Print version
```
Coral has the ability to process multiple nodes in the parse tree per folding, this is controlled by the `--batch-size` parameter. A larger batch size will require fewer total proving steps, but each step will have more constraints. In our experience, between 5 and 10 total steps is usually optimal. Depending on you tree this will be a batch size between 150 and 1,000. Performace will significantly degrade as the batch size increases beyond 2,500.

You can use `--cmt-name` and `--proof-name` to choose names for your
commitment and proof files. This is optional - Coral will choose a name for the
commitment/proof if you do not. 

## Perpetual Powers of Tau 
You will need a local copy of the [Perpetual Powers of Tau](https://github.com/privacy-scaling-explorations/perpetualpowersoftau) to run Coral. Coral is hardcoded to use **./ppot_0080_23.ptau*. However, you can use whichever one you prefer by changing the specified file [here](https://github.com/eniac/coral/blob/main/src/solver.rs#L841) and [here](https://github.com/eniac/coral/blob/main/src/util.rs#L129).

## Sample Grammars
The grammars directory contains sample grammars for a JSON, TOML, and a subset of C. You can run Coral for JSON with the following
```
./target/release/coral -d ./tests/test_docs/json/test_json_64.txt -g ./grammars/json.pest -b 100 -m ./tests/results/timings/scale/json_64_coral.txt --e2e
```


## Reproducing Baseline Results
If you're interested in reproducing our baseline results, you can run the corresponding scripts in the **tests/scripts** directory. We have also provided a python notebook **DataCleaning** to help reproduce our analysis. 

Thank you for using Coral,
Happy proving!
