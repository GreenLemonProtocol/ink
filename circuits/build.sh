
export PATH="${HOME}/.zokrates/bin:$PATH"
zokrates compile -i withdraw.zok
zopatract setup
mv abi.json out proving.key verification.key ../build