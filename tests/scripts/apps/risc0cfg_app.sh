mkdir -p tests/results/memory 
mkdir -p tests/results/timings

cd risc0/cfg_gen

cargo clean 
cargo build --release --features 'metrics' 

# for i in {0..1}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_citi_risc0_cfg ./target/release/validate_input ../../tests/test_docs/json/bank_citi.txt ../../grammars/json.pest ../../tests/results/timings/json_citi_risc0_cfg.txt
# done

# for i in {0..1}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_plaid_risc0_cfg ./target/release/validate_input ../../tests/test_docs/json/bank_plaid.txt ../../grammars/json.pest ../../tests/results/timings/json_plaid_risc0_cfg.txt
# done 

# for i in {0..1}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_dmv_veratad_risc0_cfg ./target/release/validate_input ../../tests/test_docs/json/dmv_veratad.txt ../../grammars/json.pest ../../tests/results/timings/json_dmv_veratad_risc0_cfg.txt
# done 

# for i in {0..1}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_dk_risc0_cfg ./target/release/validate_input ../../tests/test_docs/json/draftgroups_dk.txt ../../grammars/json.pest ../../tests/results/timings/json_dk_risc0_cfg.txt
# done


# for i in {0..1}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_hibp_small_risc0_cfg ./target/release/validate_input ../../tests/test_docs/json/hibp_small.txt ../../grammars/json.pest ../../tests/results/timings/json_hibp_small_risc0_cfg.txt
# done

# for i in {0..1}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_hibp_risc0_cfg ./target/release/validate_input ../../tests/test_docs/json/hibp.txt ../../grammars/json.pest ../../tests/results/timings/json_hibp_risc0_cfg.txt
# done

for i in {0..1}
do 
echo "$i"
RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/apps/json_jwt_risc0_cfg ./target/release/validate_input ../../tests/test_docs/json/jwt.txt ../../grammars/json.pest ../../tests/results/timings/apps/json_jwt_risc0_cfg.txt
done

# for i in {0..1}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/toml_ark_risc0_cfg ./target/release/validate_input ../../tests/test_docs/toml/arkr1cs_toml.txt ../../grammars/toml.pest ../../tests/results/timings/toml_ark_risc0_cfg.txt
# done

# for i in {0..1}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/toml_coral_risc0_cfg ./target/release/validate_input ../../tests/test_docs/toml/coral_toml.txt ../../grammars/toml.pest ../../tests/results/timings/toml_coral_risc0_cfg.txt
# done

# for i in {0..1}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/toml_small_risc0_cfg ./target/release/validate_input ../../tests/test_docs/toml/small_toml.txt ../../grammars/toml.pest ../../tests/results/timings/toml_small_risc0_cfg.txt
# done

# for i in {0..1}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/c_c1_risc0_cfg ./target/release/validate_input ../../tests/test_docs/c/c1.txt ../../grammars/c_simple.pest ../../tests/results/timings/c_c1_risc0_cfg.txt
# done

# for i in {0..1}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/c_c2_risc0_cfg ./target/release/validate_input ../../tests/test_docs/c/c2.txt ../../grammars/c_simple.pest ../../tests/results/timings/c_c2_risc0_cfg.txt
# done

# for i in {0..1}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/c_c3_risc0_cfg ./target/release/validate_input ../../tests/test_docs/c/c3.txt ../../grammars/c_simple.pest ../../tests/results/timings/c_c3_risc0_cfg.txt
# done

# for i in {0..1}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/c_llvm_risc0_cfg ./target/release/validate_input ../../tests/test_docs/c/llvm_test_puzzle.txt ../../grammars/c_simple.pest ../../tests/results/timings/c_llvm_risc0_cfg.txt
# done
