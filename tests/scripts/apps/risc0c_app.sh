mkdir -p tests/results/memory 
mkdir -p tests/results/timings

cd risc0/c

cargo clean 
cargo build --release --features 'metrics' 

for i in {0..10}
do 
echo "$i"
RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/apps/c_c1_risc0_c ./target/release/c-example ../../tests/test_docs/c/c1.txt ../../tests/results/timings/apps/c_c1_risc0_c.txt
done

for i in {0..10}
do 
echo "$i"
RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/apps/c_c2_risc0_c ./target/release/c-example ../../tests/test_docs/c/c2.txt ../../tests/results/timings/apps/c_c2_risc0_c.txt
done

for i in {0..10}
do 
echo "$i"
RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/apps/c_c3_risc0_c ./target/release/c-example ../../tests/test_docs/c/c3.txt ../../tests/results/timings/apps/c_c3_risc0_c.txt
done

for i in {0..10}
do 
echo "$i"
RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/apps/c_llvm_risc0_c ./target/release/c-example ../../tests/test_docs/c/llvm_test_puzzle_mod.txt ../../tests/results/timings/apps/c_llvm_risc0_c.txt
done

