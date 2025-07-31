mkdir -p tests/results/memory 
mkdir -p tests/results/timings

cd risc0/json

cargo clean 
cargo build --release --features 'metrics' 

# for i in {0..10}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_citi_risc0_json ./target/release/json-example ../../tests/test_docs/json/bank_citi.txt ../../tests/results/timings/json_citi_risc0_json.txt
# done

# for i in {0..10}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_plaid_risc0_json ./target/release/json-example ../../tests/test_docs/json/bank_plaid.txt ../../tests/results/timings/json_plaid_risc0_json.txt
# done 

# for i in {0..10}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_dmv_veratad_risc0_json ./target/release/json-example ../../tests/test_docs/json/dmv_veratad.txt ../../tests/results/timings/json_dmv_veratad_risc0_json.txt
# done 

# for i in {0..10}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_dk_risc0_json ./target/release/json-example ../../tests/test_docs/json/draftgroups_dk.txt ../../tests/results/timings/json_dk_risc0_json.txt
# done


# for i in {0..10}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_hibp_small_risc0_json ./target/release/json-example ../../tests/test_docs/json/hibp_small.txt ../../tests/results/timings/json_hibp_small_risc0_json.txt
# done

# for i in {0..10}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_hibp_risc0_json ./target/release/json-example ../../tests/test_docs/json/hibp.txt ../../tests/results/timings/json_hibp_risc0_json.txt
# done

for i in {0..10}
do 
echo "$i"
RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/apps/json_jwt_risc0_json ./target/release/json-example ../../tests/test_docs/json/jwt.txt ../../tests/results/timings/apps/json_jwt_risc0_json.txt
done