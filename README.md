# Green Lemon Protocol

The Green Lemon Protocol is an anonymous decentralized application platform based on zero-knowledge proof and dual-key stealth address protocol. Anyone can deploy their own anonymous smart contract and share the same relayer service. And NFT(ERC-721) is supported for now, ERC-20 is under development.

This project is funded by the Web3 Foundation Grants Program.

* [Proposal: Dual-Key Stealth Address Protocol](https://github.com/w3f/Grants-Program/pull/997)
* [Proposal: Green Lemon Protocol🍋 - An anonymous NFT solution](https://github.com/w3f/Grants-Program/pull/1096)

Quick facts:
* Anonymous NFT contract hiding token owners based on [dual-key stealth address protocol](https://github.com/GreenLemonProtocol/dksap-polkadot)
* Send anonymous transactions to relayer contract based on zero-knowledge proof

The relayer contract contains three functions: deposit, withdrawal, and execute.

* Deposit: The user deposit a coin to the NFT anonymous contract and get a note, which is used to pay the relayer fees for anonymous transactions.
* Withdrawal: The user takes back the coin previously deposited, and nullifies the corresponding note. 
* Execute: The user calls the NFT contract's function through the relayer contract.

Both `withdrawal` and `execute` require the user generate a zero-knowledge proof. The `withdrawal` will send the coin back to the user. The `execute` will transfer coin to the relayer as transaction fees.

Medium articles:

* [Green Lemon Protocol — An anonymous NFT solution](https://medium.com/@wuyahuang/green-lemon-protocol-an-anonymous-nft-solution-2fad91cc8f48)

### Work flow

![workflow.jpg](./docs/workflow.jpg)

### Install
If you are a new talent for Polkadot blockchain or Node.js, please install the DEV environment first.

* Download [substrate-greenlemon-node](https://github.com/GreenLemonProtocol/substrate-contracts-node/releases). The substrate-greenlemon-node is a new version of the substrate after increased MAXIMUM_BLOCK_WEIGHT. The default value of MAXIMUM_BLOCK_WEIGHT is too low to verify zero-knowledge proof on-chain.
* [Install Node.js environment](https://nodejs.org/en/download/)
* [Install Zokrates](https://zokrates.github.io/gettingstarted.html)
* [Install cargo-contract](https://github.com/paritytech/cargo-contract), because we need to add nightly builds to Rust runtime env & install binaryen in a version >= 99.
* Install test dependencies. `npm install -d`

### Contract build & test

Contract relayer & Contract verifier

```
cd contracts
sh ./build-test-all.sh
```

#### Generate docs

```
cargo doc --open
```

### Test contract on-chain

#### Deploy contract to local node

#### 1. Start the local substrate node
```
./substrate-greenlemon-node --dev
```

#### 2. Deploy compiled contract `erc721` and `verifier` and `relayer` to local node by [Polkadot/Substrate Portal](https://polkadot.js.org/apps/#/explorer).

#### 3. Update contract address
Copy `relayer` contract address from `Polkadot/Substrate Portal` after contract deployed, open http/config/default.json, and update `ContractAddress`.


### Generate commitment and proof

#### 1.Generate commitment

```
node tests/1-generateCommitment.js
```

#### 2.Compute witness

```
node tests/2-compute-witness.js
```

#### 3.Generate zero knowledge proof

```
node tests/3-generate-proof.js
```

#### 4.Verify zero knowledge proof off-chain

```
node tests/4-verify-proof-offchain.js
```
