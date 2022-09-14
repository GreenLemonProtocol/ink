
export PATH="${HOME}/.zokrates/bin:$PATH"
zokrates compile -i withdraw.zok
zokrates setup
mv abi.json out proving.key verification.key ../build
