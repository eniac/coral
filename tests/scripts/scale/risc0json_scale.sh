mkdir -p tests/results/memory 
mkdir -p tests/results/timings

cd risc0/json

cargo clean 
cargo build --release --features 'metrics' 

# echo "64" 
# for i in {0..10}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_64_risc0_json ./target/release/json-example ../../tests/test_docs/json/test_json_64.txt ../../tests/results/timings/json_64_risc0_json.txt
# done

# echo "128" 
# for i in {0..10}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_128_risc0_json ./target/release/json-example ../../tests/test_docs/json/test_json_128.txt ../../tests/results/timings/json_128_risc0_json.txt
# done 

# echo "256"
# for i in {0..10}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_256_risc0_json ./target/release/json-example ../../tests/test_docs/json/test_json_256.txt ../../tests/results/timings/json_256_risc0_json.txt
# done 

# echo "512"
# for i in {0..10}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_512_risc0_json ./target/release/json-example ../../tests/test_docs/json/test_json_512.txt ../../tests/results/timings/json_512_risc0_json.txt
# done


# echo "1024"
# for i in {0..10}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_1024_risc0_json ./target/release/json-example ../../tests/test_docs/json/test_json_1024.txt ../../tests/results/timings/json_1024_risc0_json.txt
# done

# echo "2048"
# for i in {0..10}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_2048_risc0_json ./target/release/json-example ../../tests/test_docs/json/test_json_2048.txt ../../tests/results/timings/json_2048_risc0_json.txt
# done


# echo "4096"
# for i in {0..10}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_4096_risc0_json ./target/release/json-example ../../tests/test_docs/json/test_json_4096.txt ../../tests/results/timings/json_4096_risc0_json.txt
# done


# echo "8192"
# for i in {0..10}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_8192_risc0_json ./target/release/json-example ../../tests/test_docs/json/test_json_8192.txt ../../tests/results/timings/json_8192_risc0_json.txt
# done

# echo "16384"
# for i in {0..10}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_16384_risc0_json ./target/release/json-example ../../tests/test_docs/json/test_json_16384.txt ../../tests/results/timings/json_16384_risc0_json.txt
# done


# echo "32768"
# for i in {0..10}
# do 
# echo "$i"
# RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/json_32768_risc0_json ./target/release/json-example ../../tests/test_docs/json/test_json_32768.txt ../../tests/results/timings/json_32768_risc0_json.txt
# done

echo "65536"
for i in {0..10}
do 
echo "$i"
RUST_BACKTRACE=1 RISC0_DEV_MODE=0 gtime -v -a -o ../../tests/results/memory/scale/json_65536_risc0_json ./target/release/json-example ../../tests/test_docs/json/test_json_65536.txt ../../tests/results/timings/scale/json_65536_risc0_json.txt
done




