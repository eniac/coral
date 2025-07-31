mkdir -p tests/results/memory 
mkdir -p tests/results/timings

cd risc0/toml

cargo clean 
cargo build --release --features 'metrics' 

for i in {0..10}
do 
echo "$i"
RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/toml_ark_risc0_toml ./target/release/toml-example ../../tests/test_docs/toml/arkr1cs_toml.txt ../../tests/results/timings/toml_ark_risc0_toml.txt
done

for i in {0..10}
do 
echo "$i"
RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/toml_coral_risc0_toml ./target/release/toml-example ../../tests/test_docs/toml/coral_toml.txt ../../tests/results/timings/toml_coral_risc0_toml.txt
done

for i in {0..10}
do 
echo "$i"
RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/toml_small_risc0_toml ./target/release/toml-example ../../tests/test_docs/toml/small_toml.txt ../../tests/results/timings/toml_small_risc0_toml.txt
done
