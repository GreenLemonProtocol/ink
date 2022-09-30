# Green Lemon Protocol

The Green Lemon Protocol is an anonymous NFT platform based on zero-knowledge proof and dual-key stealth address protocol. Anyone can deploy their own anonymous NFT contract and share the same relayer service.

The platform contains two core contracts. Anonymous NFT contract and relayer contract.

Quick facts:
* Anonymous NFT contract hiding token owners based on dual-key stealth address protocol
* Send anonymous transactions to relayer contract based on zero-knowledge proof

Click [The DKSAP implementation for the Polkadot ecosystem](https://github.com/GreenLemonProtocol/dksap-polkadot) to read more information about anonymous NFT contract.

The relayer contract contains three functions: deposit, withdrawal, and execute.

* Deposit: The user deposit a coin to the NFT anonymous contract and get a note, which is used to pay the relayer fees for anonymous transactions.
* Withdrawal: The user takes back the coin previously deposited, and nullifies the corresponding note. 
* Execute: The user calls the NFT contract's function through the relayer contract.

Both `withdrawal` and `execute` require the user generate a zero-knowledge proof. The `withdrawal` will send the coin back to the user. The `execute` will transfer notes to the relayer as transaction fees.

## Build circuits

```
sh circuits/build.sh
```

## Build contract

```
cd contracts/anonymous
cargo build --release

cd ../verifier
cargo build --release
```

## Test contract

```
cargo +nightly contract test
```

## Deploy contract

1. deploy verifier contract

2. deploy anonymous contract with verifier contract account id

## Generate docs

```
cargo doc --open
```

## Generate commitment

```
node tests/index.js
```

## Generate proof by zokrates

1、compute witness

```
node tests/index.js
```

copy `witness inputs`

```
cd build
zokrates compute-witness -a COPY_WITNESS_INPUTS
```

2、generate proof

```
zokrates generate-proof
```
