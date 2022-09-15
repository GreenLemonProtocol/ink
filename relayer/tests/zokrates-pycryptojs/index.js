const pyScriptExec = require('./src/pythonShellExec');

const keygen = async () => {
  const input = [ 'keygen' ];
  const scriptExecOut = await pyScriptExec.pyScriptExec('cli.py', input);
  return scriptExecOut;
}

const sign = async ( privateKey, message ) => {
  const input = [ 'sig-gen', privateKey, message ];
  const scriptExecOut = await pyScriptExec.pyScriptExec('cli.py', input);
  return scriptExecOut;
}

const verify = async ( publicKey, message, signature_R, signature_S ) => {
  const input = [ 'sig-verify', publicKey, message, signature_R, signature_S ];
  const scriptExecOut = await pyScriptExec.pyScriptExec('cli.py', input);
  return scriptExecOut;
}

const pedersenHash = async (message, size) => {
  const input = [ 'hash', message, '-s', size];
  const scriptExecOut = await pyScriptExec.pyScriptExec('cli.py', input);
  return scriptExecOut;
}

module.exports = {
  keygen,
  sign,
  verify,
  pedersenHash,
}
