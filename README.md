# Green Lemon Protocol

The Green Lemon Protocol is an anonymous decentralized application platform based on zero-knowledge proof and dual-key stealth address protocol. Anyone can deploy their own anonymous smart contract and share the same relayer service. And NFT(ERC-721) is supported for now, ERC-20 is under development.

The platform contains two core contracts for now. Anonymous NFT contract and relayer contract.

Quick facts:
* Anonymous NFT contract hiding token owners based on dual-key stealth address protocol
* Send anonymous transactions to relayer contract based on zero-knowledge proof

Click [The DKSAP implementation for the Polkadot ecosystem](https://github.com/GreenLemonProtocol/dksap-polkadot) to read more information about anonymous NFT contract.

The relayer contract contains three functions: deposit, withdrawal, and execute.

* Deposit: The user deposit a coin to the NFT anonymous contract and get a note, which is used to pay the relayer fees for anonymous transactions.
* Withdrawal: The user takes back the coin previously deposited, and nullifies the corresponding note. 
* Execute: The user calls the NFT contract's function through the relayer contract.

Both `withdrawal` and `execute` require the user generate a zero-knowledge proof. The `withdrawal` will send the coin back to the user. The `execute` will transfer notes to the relayer as transaction fees.

## Install
If you are a new talent for Polkadot blockchain or Node.js, please install the environment first.

[Download substrate-greenlemon-node](https://github.com/GreenLemonProtocol/substrate-contracts-node/releases)

The substrate-greenlemon-node is a new version of the substrate after increased MAXIMUM_BLOCK_WEIGHT. The default value of MAXIMUM_BLOCK_WEIGHT is too low to verify zero-knowledge proof on-chain.

[Install Node.js environment](https://nodejs.org/en/download/)

[Install Zokrates](https://zokrates.github.io/gettingstarted.html)

Please [install cargo-contract](https://github.com/paritytech/cargo-contract) before build contracts, because we need to add nightly builds to runtime env & install binaryen in a version >= 99.


```
# Install project dependencies
npm install -d
```

## Contract build & test

```
cd contracts
cargo build --manifest-path relayer/Cargo.toml
cargo test --manifest-path relayer/Cargo.toml

cargo build --manifest-path verifier/Cargo.toml
cargo test --manifest-path verifier/Cargo.toml
```

#### Generate docs

```
cargo doc --open
```

## Deploy contract

Upload compiled contract `relayer` and `verifier` to local node by [Polkadot/Substrate Portal](https://polkadot.js.org/apps/#/explorer).

## Test
#### Generate commitment

```
node tests/1-generateCommitment.js
```

#### Compute witness

```
node tests/2-compute-witness.js
```

#### Generate zero knowledge proof

```
node tests/3-generate-proof.js
```

#### Verify zero knowledge proof off-chain

```
node tests/4-verify-proof-offchain.js
```
