const crypto = require('crypto');
const fs = require('fs');
const merkleTree = require('./merkle-tree.js');
const polkadotCrypto = require('@polkadot/util-crypto');
const { initialize } = require('zokrates-js');
const { assert } = require('console');
const { mimcHash } = require('./mimc');
const bigInt = require('big-integer');
const path = require('path');
// merkle tree levels
const LEVEL = 10;

(async () => {
  // zokrates field value can only hold 254 bits
  let nullifier = crypto.randomBytes(31);
  // let nullifier = Buffer.from([
  //   0x47, 0x14, 0x24, 0xa3, 0xbb, 0x44, 0x1f, 0xde, 0x5e, 0x66, 0x07, 0x1c,
  //   0x0d, 0x74, 0xba, 0xc7, 0x94, 0xd7, 0x00, 0xcb, 0x8d, 0xbb, 0x8f, 0x1a,
  //   0x99, 0x63, 0x60, 0x87, 0x0b, 0xc6, 0xae,
  // ]);
  let secret = crypto.randomBytes(31);
  // let secret = Buffer.from([
  //   0x4d, 0x95, 0x45, 0xe1, 0x51, 0x7b, 0x6e, 0x70, 0x35, 0x7d, 0x09, 0xd9,
  //   0x61, 0x76, 0xe2, 0x17, 0x73, 0x06, 0x6a, 0xf8, 0x3d, 0x22, 0x3a, 0xd3,
  //   0xb7, 0x5d, 0x3a, 0xaf, 0x2d, 0x9b, 0x51,
  // ]);
  let commitment = mimcHash(rbigint(nullifier), rbigint(secret));
  // construct a merkle tree, it contains one leaf for this example
  let leaves = [commitment];
  let tree = new merkleTree();
  tree.init(LEVEL, leaves, { hashFunction: mimcHash });
  // console.log(tree.root().toString())
  // return
  let nullifierHash = mimcHash(rbigint(nullifier));

  let index = 0;
  const { pathElements, pathIndices } = tree.path(index);
  let pathE = [[], [], []];
  for (let i = 0; i < LEVEL; ++i) {
    pathE[i] = pathElements[i].toString();
  }
  pathE = pathE.flat(Infinity);
  let recipient = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
  let receiptArray = polkadotCrypto.decodeAddress(recipient);
  let relayer = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty';
  let relayerArray = polkadotCrypto.decodeAddress(relayer);
  let fee = '1000000000';
  let refund = '2000000000';
  for (let i = 0; i < pathIndices.length; ++i) {
    pathIndices[i] = pathIndices[i].toString();
  }

  // inputs for compute witness
  let inputs = [
    // public input
    tree.root().toString(),
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
  console.log("witness inputs:\n" + inputs.toString().replace(/,/g, ' '));
  console.log("commitment:", commitment.toString(16));

  // public inputs for ink! contract to withdraw
  console.log('root:', tree.root().toString(16));
  console.log('nullifierHash:', nullifierHash.toString(16));
  console.log('recipient:', recipient);
  console.log('relayer:', relayer);
  console.log('fee:', fee);
  console.log('refund:', refund);
  return;

  ///////////////////////////////
  // Generate Proof
  ///////////////////////////////
  // initialize zokrates provider
  let zokratesProvider = await initialize();
  const artifacts = {
    program: Array.from(fs.readFileSync(path.join(__dirname + '/../build/out'))),
    abi: fs.readFileSync(path.join(__dirname + '/../build/abi.json')).toString(),
  };

  console.log('\nstart generating zero-knowledge proof, it takes about 1 minute.\n');
  // compute witness
  const { witness } = zokratesProvider.computeWitness(artifacts, inputs);

  // generate zk proof
  let provingKey = Array.from(fs.readFileSync(path.join(__dirname + '/../build/proving.key')));
  const proofAndInput = zokratesProvider.generateProof(
    artifacts.program,
    witness,
    provingKey
  );

  // zk proof verify
  const verificationKey = JSON.parse(
    fs.readFileSync(path.join(__dirname + '/../build/verification.key'))
  );
  const isVerified = zokratesProvider.verify(verificationKey, proofAndInput);
  assert(isVerified);
  // console.log(proofAndInput);

  // proof for ink! contract to withdraw
  console.log(
    'proof:',
    to_g1(proofAndInput.proof.a) +
    to_g2(proofAndInput.proof.b) +
    to_g1(proofAndInput.proof.c)
  );
  // public inputs for ink! contract to withdraw
  console.log('root:', tree.root().toString(16));
  console.log('nullifierHahs:', nullifierHash.toString(16));
  console.log('recipient:', recipient);
  console.log('relayer:', relayer);
  console.log('fee:', fee);
  console.log('refund:', refund);
})();

// decode hex to Buffer and reverse
function decodeHex(value) {
  return Buffer.from(value.replace('0x', ''), 'hex').reverse();
}

// encode zk proof a and c to hex string
function to_g1(g1) {
  let buf = Buffer.concat([
    decodeHex(g1[0]),
    decodeHex(g1[1]),
    Buffer.from([0]),
  ]);
  return buf.toString('hex');
}

// encode zk proof b to hex string
function to_g2(g2) {
  let buf = Buffer.concat([
    decodeHex(g2[0][0]),
    decodeHex(g2[0][1]),
    decodeHex(g2[1][0]),
    decodeHex(g2[1][1]),
    Buffer.from([0]),
  ]);
  return buf.toString('hex');
}

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