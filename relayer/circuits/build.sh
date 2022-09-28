
export PATH="${HOME}/.zokrates/bin:$PATH"
cd `dirname $0`
zokrates compile -i withdraw.zok
mv abi.json out ../build && rm out.r1cs

###  if you want to change verification.key and proving.key 
# zokrates setup
# mv proving.key verification.key ../build
