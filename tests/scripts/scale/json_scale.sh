# mkdir -p ./tests/results/memory 
# mkdir -p ./tests/results/timings

cargo clean 
cargo build --release --features 'metrics para' 

echo "64" 
declare -a b_64=(40 47 59 79 118)
for i in {0..10}
do 
echo "$i"
for j in "${b_64[@]}"
do 
./target/release/coral -d ./tests/test_docs/json/test_json_64.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_64_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_64_coral ./target/release/coral -d ./tests/test_docs/json/test_json_64.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_64_coral.txt --prove

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_64_coral ./target/release/coral -d ./tests/test_docs/json/test_json_64.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_64_coral.txt --verify
done 
done

echo "128" 
declare -a b_128=(58 68 81 101 135)
for i in {0..10}
do 
echo "$i"
for j in "${b_128[@]}"
do
./target/release/coral -d ./tests/test_docs/json/test_json_128.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_128_coral.txt --commit 

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_128_coral ./target/release/coral -d ./tests/test_docs/json/test_json_128.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_128_coral.txt --prove

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_128_coral ./target/release/coral -d ./tests/test_docs/json/test_json_128.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_128_coral.txt --verify
done 
done 

echo "256"
declare -a b_256=(137 159 191 239 318)
for i in {0..10}
do 
echo "$i"
for j in "${b_256[@]}"
do 
./target/release/coral -d ./tests/test_docs/json/test_json_256.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_256_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_256_coral ./target/release/coral -d ./tests/test_docs/json/test_json_256.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_256_coral.txt --prove

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_256_coral ./target/release/coral -d ./tests/test_docs/json/test_json_256.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_256_coral.txt --verify
done
done 

echo "512"
declare -a b_512=(276 315 368 441 552)
for i in {0..10}
do 
echo "$i"
for j in "${b_512[@]}"
do 
./target/release/coral -d ./tests/test_docs/json/test_json_512.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_512_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_512_coral ./target/release/coral -d ./tests/test_docs/json/test_json_512.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_512_coral.txt --prove

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_512_coral ./target/release/coral -d ./tests/test_docs/json/test_json_512.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_512_coral.txt --verify
done
done


echo "1024"
declare -a b_1024=(333 363 444 499 570)
for i in {0..10}
do 
echo "$i"
for j in "${b_1024[@]}"
do 
./target/release/coral -d ./tests/test_docs/json/test_json_1024.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_1024_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_1024_coral ./target/release/coral -d ./tests/test_docs/json/test_json_1024.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_1024_coral.txt --prove

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_1024_coral ./target/release/coral -d ./tests/test_docs/json/test_json_1024.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_1024_coral.txt --verify
done
done

echo "2048"
declare -a b_2048=(615 666 726 799 887)
for i in {0..10}
do 
echo "$i"
for j in "${b_2048[@]}"
do 
./target/release/coral -d ./tests/test_docs/json/test_json_2048.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_2048_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_2048_coral ./target/release/coral -d ./tests/test_docs/json/test_json_2048.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_2048_coral.txt --prove

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_2048_coral ./target/release/coral -d ./tests/test_docs/json/test_json_2048.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_2048_coral.txt --verify
done
done


echo "4096"
declare -a b_4096=(735 770 899 952 1115)
for i in {0..10}
do 
echo "$i" 
for j in "${b_4096[@]}"
do 
./target/release/coral -d ./tests/test_docs/json/test_json_4096.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_4096_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_4096_coral ./target/release/coral -d ./tests/test_docs/json/test_json_4096.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_4096_coral.txt --prove

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_4096_coral ./target/release/coral -d ./tests/test_docs/json/test_json_4096.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_4096_coral.txt --verify
done
done

echo "8192"
declare -a b_8192=(1040 1170 1221 1277 2006)
for i in {0..10}
do 
echo "$i"
for j in "${b_8192[@]}"
do
./target/release/coral -d ./tests/test_docs/json/test_json_8192.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_8192_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_8192_coral ./target/release/coral -d ./tests/test_docs/json/test_json_8192.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_8192_coral.txt --prove

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_8192_coral ./target/release/coral -d ./tests/test_docs/json/test_json_8192.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_8192_coral.txt --verify
done
done

echo "16384"
declare -a b_16384=(2102 2264 2354 2452 2559)
for i in {0..10}
do
echo "$i"
for j in "${b_16384[@]}"
do
./target/release/coral -d ./tests/test_docs/json/test_json_16384.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_16384_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_16384_coral ./target/release/coral -d ./tests/test_docs/json/test_json_16384.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_16384_coral.txt --prove

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_16384_coral ./target/release/coral -d ./tests/test_docs/json/test_json_16384.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_16384_coral.txt --verify
done
done

echo "32768"
declare -a b_32768=(1199 1618 2122 2232 2397)
for i in {0..5}
do 
echo "$i"
for j in "${b_32768[@]}"
do
./target/release/coral -d ./tests/test_docs/json/test_json_32768.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_32768_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_32768_coral ./target/release/coral -d ./tests/test_docs/json/test_json_32768.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_32768_coral.txt --prove

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_32768_coral ./target/release/coral -d ./tests/test_docs/json/test_json_32768.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_32768_coral.txt --verify
done
done

echo "65536"
declare -a b_65536=(1594 1999 2183 2384 2428)
for i in {0..10}
do 
echo "$i"
for j in "${b_65536[@]}"
do
./target/release/coral -d ./tests/test_docs/json/test_json_65536.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_65536_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_65536_coral ./target/release/coral -d ./tests/test_docs/json/test_json_65536.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_65536_coral.txt --prove

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/scale/json_65536_coral ./target/release/coral -d ./tests/test_docs/json/test_json_65536.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/scale/json_65536_coral.txt --verify
done
done