// Import
import axios from 'axios';
import nconf from 'nconf';
import * as secp256k1 from '@noble/secp256k1';
import { contractQuery, generateEncyptedAddress, bytesToHex } from './util.js';

const aliceAlias = 'Alice';

try {
  // Read constants from config
  nconf.file('./config/default.json');
  const RelayerServiceAddress = nconf.get('RelayerServiceAddress');

  // Query Alice public keys
  const alicePublicKeys = await contractQuery('publicKeysOf', aliceAlias);
  console.log('Alice public keys: ' + alicePublicKeys);

  // Convert hex to elliptic curve point
  const scanPublicKeyPoint = secp256k1.Point.fromHex(alicePublicKeys[0]);
  const spendPublicKeyPoint = secp256k1.Point.fromHex(alicePublicKeys[1]);

  // Generate Encrypted address by Alice's public keys
  const { ephemeralPublicKey, owner } = await generateEncyptedAddress(scanPublicKeyPoint, spendPublicKeyPoint);

  // Compress ephemeral public key
  let ephemeralPublicKeyBytes = ephemeralPublicKey.toRawBytes(true);

  nconf.file('./proofs/proof-1.json');
  const nullifierHash = nconf.get('nullifierHash');

  // Ensure note is not spent
  const isHashNullified = await contractQuery('isHashNullified', nullifierHash);

  if (!isHashNullified) {
    // Send transaction through relayer service
    let res = await axios({
      url: RelayerServiceAddress,
      method: 'post',
      timeout: 10000,
      data: {
        action: 'execute',
        proof: nconf.get('proof'),
        root: nconf.get('root'),
        nullifierHash: nullifierHash,
        recipient: nconf.get('recipient'),
        relayer: nconf.get('relayer'),
        fee: nconf.get('fee'),
        refund: nconf.get('refund'),
        function: 'mint',
        selector: '0xcfdd9aa2',
        contract_params: [{ 'accountid': owner }, { 'string': bytesToHex(ephemeralPublicKeyBytes) }]
      },
      headers: {
        'Content-Type': 'application/json',
      }
    });

    // Check status of relayer repsonse
    if (res.status == 200) {
      console.log('Transaction sent with hash ' + res.data);
      console.log('Encrypted destination address: ' + owner);
    } else {
      console.log('Transaction sent failed, please check your connection to relayer service.');
    }
  } else {
    console.log('Current zero-knowledge proof is already spent');
  }

  process.exit();
} catch (error) {
  console.log("Send Transaction failed: " + error);
}