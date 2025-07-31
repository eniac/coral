
# cargo clean 
# cargo build --release --features 'metrics para' 


echo "citi" 
declare -a b_citi=(575 611 652 698 752 814)
for i in {0..10}
do 
echo "$i"
for j in "${b_citi[@]}"
do
./target/release/coral -d ./tests/test_docs/json/bank_citi.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps/json_bank_citi_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps/json_bank_citi_coral ./target/release/coral -d ./tests/test_docs/json/bank_citi.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps/json_bank_citi_coral.txt --prove
done 
done 

echo "plaid" 
declare -a b_plaid=(109 118 128 141 157 176 201)
for i in {0..10}
do 
echo "$i"
for j in "${b_plaid[@]}"
do
./target/release/coral -d ./tests/test_docs/json/bank_plaid.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps/json_bank_plaid_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps/json_bank_plaid_coral ./target/release/coral -d ./tests/test_docs/json/bank_plaid.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps/json_bank_plaid_coral.txt --prove
done 
done 

echo "veratad" 
declare -a b_dmv=(249 274 305)
for i in {0..10}
do 
echo "$i"
for j in "${b_dmv[@]}"
do
./target/release/coral -d ./tests/test_docs/json/dmv_veratad.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps/json_dmv_veratad_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps/json_dmv_veratad_coral ./target/release/coral -d ./tests/test_docs/json/dmv_veratad.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps/json_dmv_veratad_coral.txt --prove
done 
done 

echo "dk" 
declare -a b_dk=(592 641 699 769 854)
for i in {0..10}
do 
echo "$i"
for j in "${b_dk[@]}"
do
./target/release/coral -d ./tests/test_docs/json/draftgroups_dk.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps/json_dk_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps/json_dk_coral ./target/release/coral -d ./tests/test_docs/json/draftgroups_dk.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps/json_dk_coral.txt --prove
done 
done 

echo "hibps" 
declare -a b_hs=(106 119 136 158 190)
for i in {0..10}
do 
echo "$i"
for j in "${b_hs[@]}"
do
./target/release/coral -d ./tests/test_docs/json/hibp_small.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps/json_hibp_small_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps/json_hibps_coral ./target/release/coral -d ./tests/test_docs/json/hibp_small.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps/json_hibp_small_coral.txt --prove
done 
done 

# echo "hibpl" 
# declare -a b_h=(400 800 1200 1806 2107)
# # for i in {0..10}
# for i in {0..5}
# do 
# echo "$i"
# for j in "${b_h[@]}"
# do
# ./target/release/coral -d ./tests/test_docs/json/hibp.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps/json_hibp_coral.txt --commit

# RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps/json_hibp_coral ./target/release/coral -d ./tests/test_docs/json/hibp.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps/json_hibp_coral.txt --prove
# done 
# done 

echo "jwt" 
declare -a b_jwt=(109 118 128 141 157 176 201)
for i in {0..10}
do 
echo "$i"
for j in "${b_jwt[@]}"
do
./target/release/coral -d ./tests/test_docs/json/jwt.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps/json_jwt_coral.txt --commit
RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps/json_jwt_coral ./target/release/coral -d ./tests/test_docs/json/jwt.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps/json_jwt_coral.txt --prove
done 
done 

