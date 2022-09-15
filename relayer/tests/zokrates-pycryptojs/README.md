# ZoKrates pyCrypto Javascript wrapper
This is javascript wrapper around [ZoKrates pyCrypto](https://github.com/Zokrates/pycrypto.git). It exposes functions to perform EdDSA on baby jub jub curve.

### Functions
The following functions can be called by importing this module
- `keygen()`
    returns: array of privateKey and publicKey


- `sign(privateKey, message)`
    returns: array of R element of signature, S element of signature


- `verify(publicKey, message, R_signature, S_signature)`
    returns: array of true on successful verification

### Example

```javascript
const pycryptoJs = require('../index.js');

const test = async () => {
  const message = '11dd22';
  const [ privateKey, publicKey ] = await pycryptoJs.keygen();
  const [ signature_R, signature_S ] = await pycryptoJs.sign(privateKey, message);
  const [ result ] = await pycryptoJs.verify(publicKey, message, signature_R, signature_S);
}

test();
```

### License
This is released under the GNU Lesser General Public License v3.
