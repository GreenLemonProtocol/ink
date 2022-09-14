# Green Lemon Protocol

## Background

Many of today’s blockchains, including Bitcoin and Ethereum, are open and public ledgers because there are no restrictions on participation and all transaction details are visible on the blockchain. In a public ledger, the transaction entities are only identified by their blockchain addresses, which are derived from the corresponding public keys. Public ledgers are generally considered to be “pseudo-anonymous”, which means that an address is linked to one person, but that person is unknown to the public. However, by analyzing the transaction graph and combining it with other information, it is possible to reveal the true real-world identity behind a blockchain address, as shown by recent research.

## Overview

The Green Lemon protocol is an anonymous NFT solution based on zero-knowledge proof and dual-key stealth address protocol: users deposit coins to an anonymous NFT contract and then anonymously send mint, transfer, and other ERC721 functions to that contract via relayer.

### Protocol Description

The protocol contains the following functions:

* Deposit: The user deposit a coin to the NFT anonymous contract and get a note, which is used to pay the relayer fees for anonymous transactions.
* Withdrawal: The user takes back the coin previously deposited, and nullifies the corresponding note.
* Registration: The user registers the Scan public key and Spend public key to the NFT contract, so other users can query it on-chain.
* Mint: The user generates the encrypted public key address ePub1(encrypted pub key) based on his scan pub and anonymously mint NFT through the relayer, the owner of this NFT is ePub1
* Transfer: The user generates the encrypted public key address ePub2 based on the scan pub of the recipient and uses the private key signature corresponding to ePub1 to anonymously transfer the NFT, and the owner of the NFT is ePub2.
* Other functions supported by ERC721.

### Protocol Algorithm

The protocol implements the function of initiating anonymous transactions through zero-knowledge proofs and the function of hiding NFT owners through DKSAP.

Currently, a large number of anonymous transaction projects use zero-knowledge proofs, such as Monero and ZCash based on the UXTO model, and Zether and Tornado based on the account model.

Zether comes to our attention with its unique implementation, which uses the Σ-Bullets protocol, does not require the generation of public parameters for the initiation ceremony, and uses the Elgamal encryption algorithm for homomorphic encryption and decryption of account balance, which are excellent features. But the Gas for anonymous transfers involving 64 accounts verified on-chain amounted to 36,152,558.

Meanwhile, Tornado, based on zkSnark, performed well in terms of Gas, with a Gas consumption of 1,088,354 for deposits and 301,233 for withdrawals, although Tornado is not designed to support cross-contract calls. After comparison, we decided to develop zero-knowledge proof module based on [zokrates](https://zokrates.github.io/gettingstarted.html)(zkSNARK).

DKSAP is a new privacy transaction protocol invented by rynomster/sdcoin in 2014. Since its announcement, it has landed in numerous blockchain projects (Monero, Samourai Wallet, TokenPay, etc.). It is characterized by the fact that the account needs to generate two sets of public and private key pairs, "scan key pair", and "spend key pair", the recipient of each transaction is encrypted and cannot be associated with a particular blockchain account.

Next, we describe the specific details of the protocol implementation.

#### Deposit

To deposit a coin, the user performs the following steps.

* Generate two random numbers nullifier, secret, calculate commitment = pedersenHash (nullifier + secret), note = format (nullifier + secret).
* Send commitment and the corresponding Token to the NFT anonymous contract, commitment will be added as a leaf node to the Merkel tree of the contract, and the root hash value of the Merkel tree will be recalculated.

After the deposit is successful, the user gets the corresponding note.

#### Send transactions related to NFT contracts.

Prepare the zero-knowledge proof circuit to input the required data:

* signal input root; 				// Merkle tree root 
* signal input nullifierHash;    // nullifier hash value
* signal input recipient;        // recipient to receive the remaining token
* signal input destination;    	// NFT receiver
* signal input relayer;          // relayer address
* signal input fee;              // transaction fee for relayer 
* signal private input nullifier;   // nullifier
* signal private input secret;      // secret
* signal private input pathElements[levels];    // path element to Merkle tree root
* signal private input pathIndices[levels];    // index of path element

Perform the following steps:

* Calculate zero-knowledge proof = statement (root, nullifierHash, recipient, destination, relayer, fee, nullifier, secret).
* Call the NFT anonymous contract through the relayer node. The contract checks whether the nullifierHash has been used, and verifies whether the proof is correct.
* Mark that nullifierHash has been spent, send the fee to the relayer, and withdraw the remaining Token to the recipient.
* Trigger the corresponding NFT function, where the fee needs to be greater than the current transaction fee.

#### DKSAP Technical Details (Dual-Key Stealth Address Protocol)

Let's take the example of Alice transferring anonymous NFT to Bob, the details are as follows.

* Bob generates two sets of public and private key pairs, scan key pair = (s, S = G ^ s ) and send key pair = (b, B = G ^ b), and publishes public keys S and B to the public.
* Alice generates a temporary set of public-private key pairs (r, R = G ^ r)
* By ECDH algorithm, Alice and Bob can compute the same shared key c = Hash(r ^ S) = Hash(r ^ G ^ s) = Hash(R ^ s) respectively
* Alice calculates P = c ^ G + B = c ^ G + b ^ G = (c + b) ^ G, where public key P is the NFT recipient and sends the public key P, R to the NFT contract along with the data of the transaction
* Bob checks the transaction data of the NFT contract in real time, gets the R of each private transaction, calculates the shared key c, and then overlays the scan key to calculate P', as long as the public key P' is equal to P in the transaction data, the receiver of the NFT is Bob himself. Since P = (c + b) ^ G, the private key corresponding to the public key P is c + b
* Only Bob owns the private key b and can also calculate the shared key c, so only Bob is the owner of the NFT

### Application Scenario

NFT sales on Ethereum exceeded $9 billion in 2021, a 25,00% increase over total sales in 2020. 2021, the year of NFT, also emerged in the context of the bull market cycle, the size of the NFT market showed an amazing growth trend, with a market capitalization of more than $10 billion. Due to the bear market in the first quarter of 2022, global NFT trading volume still rose 13.25% quarter-on-quarter.

Sotheby's - a renowned auction house with a history of nearly 300 years - generated $7.3 billion in sales in 2021, of which 10% was in private transactions. This gives us confidence that anonymous trading, the act of buying and selling without revealing the identity of the trader, is just as strongly demanded in the NFT ecosystem. If the demand for anonymous trading is 1/10 of the total demand for trading, then assuming NFT sales of $10 billion in 2022, the potential sales of anonymous NFT trading would be $1 billion.

As the first anonymous NFT application of web3 Ecology, we believe Green Lemon will have a positive impact on web3. Users of NFT who value their privacy greatly will find it attractive.
