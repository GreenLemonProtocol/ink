# Relayer Contract

The relayer contract contains three functions: deposit, withdrawal, and execute.

* Deposit: The user deposit a coin to the NFT anonymous contract and get a note, which is used to pay the relayer fees for anonymous transactions.
* Withdrawal: The user takes back the coin previously deposited, and nullifies the corresponding note. 
* Execute: The user calls the NFT contract's function through the relayer contract.

Both `withdrawal` and `execute` require the user generate a zero-knowledge proof. The `withdrawal` will send the coin back to the user. The `execute` will transfer notes to the relayer as transaction fees.

## Build circuits

```sh
sh circuits/build.sh
```

## Build contract

```sh
cd contracts/anonymous
cargo build --release

cd ../verifier
cargo build --release
```

## Test contract

```sh
cargo +nightly contract test
```

## Deploy contract

1. deploy verifier contract

2. deploy anonymous contract with verifier contract account id

## Generate docs

```sh
cargo doc --open
```

## Generate commitment

```sh
node tests/index.js
```

## Generate proof by zokrates

1、compute witness

```sh
node tests/index.js
```

copy `witness inputs`

```sh
cd build
zokrates compute-witness -a COPY_WITNESS_INPUTS
```

2、generate proof

```sh
zokrates generate-proof
```
