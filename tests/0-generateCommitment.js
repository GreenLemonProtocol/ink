import crypto from 'crypto';
import { decodeAddress } from '@polkadot/util-crypto';
// import initialize from 'zokrates-js';
// import assert from 'console';
import bigInt from 'big-integer';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

import { mimcHash } from './mimc.js';

import { MerkleTree } from 'fixed-merkle-tree'

// merkle tree levels
const LEVEL = 10;

(async () => {
  // zokrates field value can only hold 254 bits
  let nullifier = crypto.randomBytes(31);
  let secret = crypto.randomBytes(31);
  let commitment = mimcHash(rbigint(nullifier), rbigint(secret));
  // construct a merkle tree, it contains one leaf for this example
  let leaves = [commitment];
  let tree = new MerkleTree(LEVEL, leaves, { hashFunction: mimcHash });
  let nullifierHash = mimcHash(rbigint(nullifier));

  // init merkle tree
  let index = leaves.length - 1;
  const { pathElements, pathIndices } = tree.path(index);
  let pathE = [[], [], []];
  for (let i = 0; i < LEVEL; ++i) {
    pathE[i] = pathElements[i].toString();
  }
  pathE = pathE.flat(Infinity);

  // params
  let recipient = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
  let receiptArray = decodeAddress(recipient);
  let relayer = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty';
  let relayerArray = decodeAddress(relayer);
  let fee = '500000000000';
  let refund = '500000000000';

  for (let i = 0; i < pathIndices.length; ++i) {
    pathIndices[i] = pathIndices[i].toString();
  }

  // inputs for compute witness
  let inputs = [
    // public input
    tree.root.toString(),
    nullifierHash.toString(),
    ...toArray(Buffer.from(receiptArray).toString('hex'), 2),
    ...toArray(Buffer.from(relayerArray).toString('hex'), 2),
    fee,
    refund,
    // private input
    rbigint(nullifier).toString(),
    rbigint(secret).toString(),
    ...pathE,
    ...pathIndices,
  ];

  // format output data
  const commitmentHex = commitment.toString(16);
  const output = {
    "witnessInputs": inputs.toString().replace(/,/g, ' '),
    "commitment": commitment.toString(),
    "commitmentHex": commitmentHex,
    // public inputs for ink! contract to withdraw
    "root": tree.root.toString(16).padStart(64, '0'),
    "nullifierHash": nullifierHash.toString(16).padStart(64, '0'),
    "recipient": recipient,
    "relayer": relayer,
    "fee": fee,
    "refund": refund,
  }

  console.log(output);

  // Save file to config directory
  // Get current directory

  // Get current directory
  const __dirname = path.dirname(fileURLToPath(import.meta.url));
  const parentDir = path.resolve(__dirname, '..');

  const outputFile = path.resolve(parentDir, './build/commitment.json');

  fs.writeFileSync(outputFile, JSON.stringify(output));
  console.log('The commitment has been generated successfully, located in ' + outputFile);

  // ///////////////////////////////
  // // Generate Proof
  // ///////////////////////////////
  // // initialize zokrates provider
  // let zokratesProvider = await initialize();
  // const artifacts = {
  //   program: Array.from(fs.readFileSync(path.join(__dirname + '/../build/out'))),
  //   abi: fs.readFileSync(path.join(__dirname + '/../build/abi.json')).toString(),
  // };

  // console.log('\nstart generating zero-knowledge proof, it takes about 1 minute.\n');
  // // compute witness
  // const { witness } = zokratesProvider.computeWitness(artifacts, inputs);

  // // generate zk proof
  // let provingKey = Array.from(fs.readFileSync(path.join(__dirname + '/../build/proving.key')));
  // const proofAndInput = zokratesProvider.generateProof(
  //   artifacts.program,
  //   witness,
  //   provingKey
  // );

  // // zk proof verify
  // const verificationKey = JSON.parse(
  //   fs.readFileSync(path.join(__dirname + '/../build/verification.key'))
  // );
  // const isVerified = zokratesProvider.verify(verificationKey, proofAndInput);
  // assert(isVerified);
  // // console.log(proofAndInput);
})();

// // 32 bytes hex string to uint32array, contains 8 elements
// function toU32Array(hexString) {
//   let result = [];
//   for (let i = 0; i < 8; ++i) {
//     result[i] = parseInt(hexString.slice(0 + i * 8, i * 8 + 8), 16).toString();
//   }
//   return result;
// }
// 32 bytes hex string to uint8array
function toArray(hexString, size) {
  let result = [];
  let len = hexString.length / size;
  for (let i = 0; i < size; ++i) {
    result[i] = bigInt(hexString.slice(0 + i * len, i * len + len), 16).toString();
  }
  return result;
}

function rbigint(nbytes) {
  return bigInt(BigInt('0x' + toHex(nbytes)));
}

// from Buffer or BigInt to hex string
function toHex(number, length = 32) {
  const str =
    number instanceof Buffer
      ? number.toString('hex')
      : BigInt(number).toString(16);
  return str.padStart(length * 2, '0');
}