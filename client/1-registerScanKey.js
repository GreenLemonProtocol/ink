// Import
// Import
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { ContractPromise } from '@polkadot/api-contract';
import fs from 'fs';
import nconf from 'nconf';

// Read constants from config
// Query Relayer address
nconf.file('./config/default.json');

// Node connect init
const wsProvider = new WsProvider(nconf.get('WsProvider'));
const api = await ApiPromise.create({ provider: wsProvider });

// Contract instance init
const metadata = JSON.parse(fs.readFileSync(nconf.get('RelayerContractMetaData')));
const RelayerContractAddress = nconf.get('RelayerContractAddress');
const contract = new ContractPromise(api, metadata, RelayerContractAddress);
const keyring = new Keyring({ type: 'sr25519' });
const depositorAccount = keyring.addFromUri(nconf.get('DepositorAccount'));

// Query Alice key pair
nconf.file('./config/alice.json');
const aliceScanKeyPair = nconf.get("ScanKeyPair");
const aliceSpendKeyPair = nconf.get("SpendKeyPair");

// Start register process
(async function () {
  try {
    await registerScanPublicKey('Alice', aliceScanKeyPair.publicKey, aliceSpendKeyPair.publicKey);
  } catch (err) {
    console.error(err);
  } finally {
    process.exit();
  }
})();

/**
   * Register scan public key and spend public to contract
   * @param alias - alias name
   * @param scanPublicKey - scan public key
   * @param spendPublicKey - spend public key
   */
async function registerScanPublicKey(alias, scanPublicKey, spendPublicKey) {
  try {
    // Query current alias record
    const { output } = await contract.query['publicKeysOf'](RelayerContractAddress, { gasLimit: 20000000000000 }, alias);
    const aliasRecord = output.toHuman();

    if (!aliasRecord) {
      // Send transaction
      console.log('alias: ' + alias);
      console.log('scanPublicKey: ' + scanPublicKey);
      console.log('spendPublicKey: ' + spendPublicKey);
      console.log();

      const mint = await contract.tx.registerPublicKeys({ gasLimit: 20000000000000 }, alias, scanPublicKey, spendPublicKey);
      const hash = await mint.signAndSend(depositorAccount);
      const hashToHex = hash.toHex();

      console.log('Transaction sent with hash', hashToHex);
    } else {
      console.log('Public key of ' + alias + ' is already exists');
      console.log(aliasRecord);
    }
    console.log();
  } catch (error) {
    console.log("Send Transaction failed: " + error);
  };
}