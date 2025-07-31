
# cargo clean 
# cargo build --release --features 'metrics para' 


echo "ark" 
declare -a b_ark=(400 580 629 656 686 718 754 794 887)
for i in {0..10}
do 
echo "$i"
for j in "${b_ark[@]}"
do
./target/release/coral -d ./tests/test_docs/toml/arkr1cs_toml.txt -g ./grammars/toml.pest -b "$j" -m ./tests/results/timings/apps/toml_ark_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps/toml_ark_coral ./target/release/coral -d ./tests/test_docs/toml/arkr1cs_toml.txt -g ./grammars/toml.pest -b "$j" -m ./tests/results/timings/apps/toml_ark_coral.txt --prove


RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps/toml_ark_coral ./target/release/coral -d ./tests/test_docs/toml/arkr1cs_toml.txt -g ./grammars/toml.pest -b "$j" -m ./tests/results/timings/apps/toml_ark_coral.txt --verify
done 
done 

echo "coral" 
declare -a b_coral=(459 492 532 554 604 665 738 782)
for i in {0..10}
do 
echo "$i"
for j in "${b_coral[@]}"
do
./target/release/coral -d ./tests/test_docs/toml/coral_toml.txt -g ./grammars/toml.pest -b "$j" -m ./tests/results/timings/apps/toml_coral_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps/toml_coral_coral ./target/release/coral -d ./tests/test_docs/toml/coral_toml.txt -g ./grammars/toml.pest -b "$j" -m ./tests/results/timings/apps/toml_coral_coral.txt --prove

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps/toml_coral_coral ./target/release/coral -d ./tests/test_docs/toml/coral_toml.txt -g ./grammars/toml.pest -b "$j" -m ./tests/results/timings/apps/toml_coral_coral.txt --verify
done 
done 

echo "small" 
declare -a b_small=(237 267 305 356 63 55 49 88 109)
for i in {0..10}
do 
echo "$i"
for j in "${b_small[@]}"
do
./target/release/coral -d ./tests/test_docs/toml/small_toml.txt -g ./grammars/toml.pest -b "$j" -m ./tests/results/timings/apps/toml_small_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps/toml_small_coral ./target/release/coral -d ./tests/test_docs/toml/small_toml.txt -g ./grammars/toml.pest -b "$j" -m ./tests/results/timings/apps/toml_small_coral.txt --prove

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps/toml_small_coral ./target/release/coral -d ./tests/test_docs/toml/small_toml.txt -g ./grammars/toml.pest -b "$j" -m ./tests/results/timings/apps/toml_small_coral.txt --verify
done 
done 

