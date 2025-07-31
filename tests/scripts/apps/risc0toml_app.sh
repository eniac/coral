mkdir -p tests/results/memory 
mkdir -p tests/results/timings

cd risc0/toml

cargo clean 
cargo build --release --features 'metrics' 

for i in {0..10}
do 
echo "$i"
RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/toml_t3_risc0_toml ./target/release/toml-example ../../tests/test_docs/toml/t3.txt ../../tests/results/timings/toml_t3_risc0_toml.txt
done

for i in {0..10}
do 
echo "$i"
RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/toml_t2_risc0_toml ./target/release/toml-example ../../tests/test_docs/toml/t2.txt ../../tests/results/timings/toml_t2_risc0_toml.txt
done

for i in {0..10}
do 
echo "$i"
RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/toml_t1_risc0_toml ./target/release/toml-example ../../tests/test_docs/toml/t1.txt ../../tests/results/timings/toml_t1_risc0_toml.txt
done
