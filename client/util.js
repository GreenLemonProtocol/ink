// Import
import { ApiPromise, WsProvider } from '@polkadot/api';
import { ContractPromise } from '@polkadot/api-contract';
import fs from 'fs';
import nconf from 'nconf';
import * as crypto from '@polkadot/util-crypto';
import * as secp256k1 from '@noble/secp256k1';

nconf.file('./config/default.json');

// Node connect init
const wsProvider = new WsProvider(nconf.get('WsProvider'));
const api = await ApiPromise.create({ provider: wsProvider });

// Contract instance init
const metadata = JSON.parse(fs.readFileSync(nconf.get('RelayerContractMetaData')));
const RelayerContractAddress = nconf.get('RelayerContractAddress');
const contract = new ContractPromise(api, metadata, RelayerContractAddress);

/**
   * Query Contract Call Function
   * @param action - contract action name
   * @param args - query params
   */
export async function contractQuery(action, ...args) {
  // console.log('RelayerContractAddress: ' + RelayerContractAddress);
  // console.log('Action: ' + action);
  // console.log(...args);
  const { output } = await contract.query[action](RelayerContractAddress, { gasLimit: 5000000000000000 }, ...args);
  return output.toHuman();
}

/**
   * Generate encrypted address by scan public key & spend public key
   * @param scanPublicKeyPoint - elliptic curve point of scan public key
   * @param spendPublicKeyPoint - elliptic curve point of spend public key
   */
export async function generateEncyptedAddress(scanPublicKeyPoint, spendPublicKeyPoint) {
  // Generate ephemeral key pair
  const mnemonic = crypto.mnemonicGenerate();
  const seed = crypto.mnemonicToMiniSecret(mnemonic);
  const keyPair = crypto.secp256k1PairFromSeed(seed);
  // (r, R)
  const ephemeralPrivateKey = BigInt('0x' + bytesToHex(keyPair.secretKey));
  const ephemeralPublicKey = secp256k1.Point.fromPrivateKey(ephemeralPrivateKey);

  // Compute a shared secret c
  const sharedSecret = crypto.keccakAsU8a(scanPublicKeyPoint.multiply(ephemeralPrivateKey).toRawBytes());
  const cToBigInt = BigInt('0x' + bytesToHex(sharedSecret));

  // Compute encrypted Bob address
  const P = secp256k1.Point.BASE.multiply(cToBigInt).add(spendPublicKeyPoint);
  const PToU8a = crypto.blake2AsU8a((P.toRawBytes(true)));
  // console.log('ephemeralPublicKey: ' + bytesToHex(ephemeralPublicKey.toRawBytes(true)));
  // console.log('encrypted address:');
  // console.log(PToU8a);

  // Convert to substrate address format
  const owner = crypto.encodeAddress(PToU8a);

  return { ephemeralPublicKey, owner };
}

/**
   * Convert bytes to hex
   * @param bytes - bytes object
   */
export function bytesToHex(bytes) {
  return Buffer.from(bytes).toString('hex');
}

/**
   * Convert integer to bytes
   * @param integer - bytes object
   */
export function intTobytes(integer) {
  var bytes = new Uint8Array(4);
  bytes[0] = (integer >> 24) & 0xff;
  bytes[1] = (integer >> 16) & 0xff;
  bytes[2] = (integer >> 8) & 0xff;
  bytes[3] = integer & 0xff;
  return bytes;
}
