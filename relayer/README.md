# Relayer Contract

The relayer contract contains three functions: deposit, register, and execute.

* Deposit: The user deposit a coin to the NFT anonymous contract and get a note, which is used to pay the relayer fees for anonymous transactions.
* Register: The user registers the `scan public key` and `spend public key` to the relayer contract so that other users can query it on-chain. 
* Execute: The user calls the NFT contract's function through the relayer contract.

Both `register` and `execute` require the user generate zero-knowledge proof to transfer notes to the relayer as transaction fees.


##### Build contract
```
cd relayer/contracts
cargo +nightly contract build
``` 

##### Test contract
```
cargo +nightly contract test
```

##### Generate docs
```
cargo doc --open
```
