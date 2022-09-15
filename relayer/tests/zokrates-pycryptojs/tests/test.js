const pycryptoJs = require('../index.js');
const edwards = require('../src/babyjubjub/compress-decompress.js');

const test = async () => {
  const message = '11dd22';
  const [ privateKeyField, publicKeyPointX, publicKeyPointY, privateKey, publicKey ] = await pycryptoJs.keygen();
  console.log("\n\n--Keygen--\n");
  console.log("Private Key Field:\n", privateKeyField);
  console.log("Public Key X:\n", publicKeyPointX);
  console.log("Public Key Y:\n", publicKeyPointY);
  console.log("Private Key:\n", privateKey);
  console.log("Public Key:\n", publicKey);
  // console.log("Edwards Compress in Node", edwards.edwardsCompress([publicKeyPointX, publicKeyPointY]));
  // console.log("Edwards Decompress in Node", edwards.edwardsDecompress(publicKey));
  const [ signature_R_X, signature_R_Y, signature_S_Field, signature_R, signature_S ] = await pycryptoJs.sign(privateKey, message);
  console.log("\n\n--Sign--\n");
  console.log("R_X of EdDSA signature:\n", signature_R_X);
  console.log("R_Y of EdDSA signature:\n", signature_R_Y);
  console.log("S Field element of EdDSA signature:\n", signature_S_Field);
  console.log("R element of EdDSA signature:\n", signature_R);
  console.log("S element of EdDSA signature:\n", signature_S);
  const [ result ] = await pycryptoJs.verify(publicKey, message, signature_R, signature_S);
  console.log("\n\n--Verify Signature--\n");
  console.log("Result:\n", result);
}

test();
