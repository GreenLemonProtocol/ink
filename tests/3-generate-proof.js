const { exec } = require("child_process");
const fs = require('fs');
const path = require('path');

exec("cd build; zokrates generate-proof;", (error, stdout, stderr) => {
  if (error) {
    console.log(`error: ${error.message}`);
    return;
  }
  if (stderr) {
    console.log(`stderr: ${stderr}`);
    return;
  }
  console.log(`stdout: ${stdout}`);

  // Read proof from ./build/proof.json
  const fileLocation = './build/proof.json';
  const parentDir = path.resolve(__dirname, '..');
  const outputFile = path.resolve(parentDir, fileLocation);

  const proofFile = JSON.parse(fs.readFileSync(outputFile, { encoding: 'utf8' }));

  // proof for ink! contract to withdraw
  console.log(
    'proof:',
    to_g1(proofFile.proof.a) +
    to_g2(proofFile.proof.b) +
    to_g1(proofFile.proof.c)
  );
});

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