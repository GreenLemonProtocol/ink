# Green Lemon Protocol

The Green Lemon Protocol is an anonymous decentralized application platform based on zero-knowledge proof and dual-key stealth address protocol. Anyone can deploy their own anonymous smart contract and share the same relayer service. And NFT(ERC-721) is supported for now, ERC-20 is under development.

This project is funded by the Web3 Foundation Grants Program.

* [Proposal: Dual-Key Stealth Address Protocol](https://github.com/w3f/Grants-Program/pull/997)
* [Proposal: Green Lemon Protocol🍋 - An anonymous NFT solution](https://github.com/w3f/Grants-Program/pull/1096)

Quick facts:
* Anonymous NFT contract hiding token owners based on dual-key stealth address protocol
* Send anonymous transactions to relayer contract based on zero-knowledge proof

For more details about dual-key stealth address protocol, please [click me](https://github.com/GreenLemonProtocol/dksap-polkadot).

The relayer contract contains 4 core functions: Deposit, RegisterPublicKeys, Withdrawal, and Execute.

* Deposit: The user deposit a coin to the NFT anonymous contract and get a note, which is used to pay the relayer fees for anonymous transactions.
* RegisterPublicKeys: The user registers the Scan public key and Spend public key to the NFT contract, so other users can query it on-chain.
* Withdrawal: The user takes back the coin previously deposited, and nullifies the corresponding note. 
* Execute: The user calls the NFT contract's function through the relayer contract.

Both `Withdrawal` and `Execute` require the user generate a zero-knowledge proof. The `Withdrawal` will send the coin back to the user. The `Execute` will transfer coin to the relayer as transaction fees.

Medium articles:

* [Green Lemon Protocol — An anonymous NFT solution](https://medium.com/@wuyahuang/green-lemon-protocol-an-anonymous-nft-solution-2fad91cc8f48)

### Product Logic

Green Lemon Protocol is a service like coin mixers that allows users to obfuscate the origin and destination of transactions. Because of the logic of the blockchain, every transaction is public. If you have some DOT on your account, you cannot transfer it anonymously, because anybody can follow your transaction history on the blockchain. Coin mixers, like Tornado Cash, can solve this privacy problem by breaking the on-chain link between the source and the destination address by using ZKP. Most importantly, we can trigger a third-party contract when we submit the zero-knowledge proof, and pay transaction fees to relayer service. If the transaction fess you paid can cover the cost of calling a third-party contract. Then the whole logic makes sense.

If you want to anonymize one of your transactions, you have to deposit a small amount of DOT on the Green Lemon Protocol contract (ex.: 1 DOT). After a little while, you can withdraw this 1 DOT with a different account. The trick is that nobody can create a link between the depositor account and the withdrawal account. If hundreds of accounts deposit 1 DOT on one side and the other hundreds of accounts withdraw 1 DOT on the other side, then nobody will be able to follow the path where the money moves.

The technical challenge is that smart contract transactions are also public like any other transaction on the Polkadot network. This is the point where zero-knowledge proof will be relevant.

For more details about zero-knowledge proof, please [click me](https://betterprogramming.pub/understanding-zero-knowledge-proofs-through-the-source-code-of-tornado-cash-41d335c5475f).

### Work flow

![workflow.jpg](./docs/workflow.jpg)

### Install
If you are a new talent for Polkadot blockchain or Node.js, please install the DEV environment first.

* Download [substrate-greenlemon-node](https://github.com/GreenLemonProtocol/substrate-contracts-node/releases). The substrate-greenlemon-node is a new version of the substrate after increased MAXIMUM_BLOCK_WEIGHT. The default value of MAXIMUM_BLOCK_WEIGHT is too low to verify zero-knowledge proof on-chain.
* [Install Node.js environment](https://nodejs.org/en/download/)
* [Install Zokrates](https://zokrates.github.io/gettingstarted.html)
* [Install cargo-contract](https://github.com/paritytech/cargo-contract), because we need to add nightly builds to Rust runtime env & install binaryen in a version >= 99.
* Install test dependencies. `npm i -d`

### Contract build & test

Contract relayer & Contract verifier

```
cd contracts
sh ./build-all.sh
sh ./test-all.sh
```

### Test contract on-chain

#### Deploy contract to local node

#### 1. Start the local substrate node
```
./substrate-greenlemon-node --dev
```

#### 2. Deploy compiled contract `erc721` and `verifier` and `relayer` to local node by [Polkadot Portal](https://polkadot.js.org/apps/#/explorer).

contract erc721 deployment constructor param:
```
baseUri: https://raw.githubusercontent.com/GreenLemonProtocol/assets/main/nft
```

contract relayer deployment constructor param:
```
levels: 10
```

#### 3. Update contract address

* Copy `RELAYER` contract address from Polkadot Portal after contract deployed, open `http/config/default.json`, and update `RelayerContractAddress`
* Copy `ERC721` contract address from Polkadot Portal after contract deployed, open `http/config/default.json`, and update `NFTContractAddress`


#### 4. Start HTTP service
```
node http/index.js
```

#### 5. Running client test cases

```
node client/0-generateKeyPair.js
node client/1-registerScanKey.js
node client/2-mintToAlice.js
```

### Generate commitment and proof manually

#### 0. run `build.sh` to compile the circuits and setup step to generate `proving.key` and `verification.key`
```
sh ./circuits/build.sh
```

#### 1. Generate commitment

```
node scripts/1-generateCommitment.js
```

#### 2. Compute witness

```
node scripts/2-compute-witness.js
```

#### 3. Generate zero knowledge proof

```
node scripts/3-generate-proof.js
```

#### 4. Verify zero knowledge proof off-chain

```
node scripts/4-verify-proof-offchain.js
```