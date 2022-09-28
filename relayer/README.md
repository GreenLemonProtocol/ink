# Relayer Contract

The relayer contract contains three functions: deposit, register, and execute.

* Deposit: The user deposit a coin to the NFT anonymous contract and get a note, which is used to pay the relayer fees for anonymous transactions.
* Register: The user registers the `scan public key` and `spend public key` to the relayer contract so that other users can query it on-chain. 
* Execute: The user calls the NFT contract's function through the relayer contract.

Both `register` and `execute` require the user generate zero-knowledge proof to transfer notes to the relayer as transaction fees.

## Build circuits

```sh
bash circuits/build.sh
```

## Build contract

```sh
bash contracts/build.sh
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
