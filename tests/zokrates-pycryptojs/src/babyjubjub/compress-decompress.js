const utils = require('./utils.js');
const { BABYJUBJUB, ZOKRATES_PRIME } = require('./config');
const { squareRootModPrime, addMod, mulMod } = require('./number-theory');
const { modDivide } = require('./modular-division'); // TODO REPLACE WITH NPM VERSION

const one = BigInt(1);
// const { JUBJUBE, JUBJUBC, JUBJUBD, JUBJUBA, GENERATOR } = BABYJUBJUB;
const Fp = BigInt(ZOKRATES_PRIME); // the prime field used with the curve E(Fp)
// const Fq = JUBJUBE / JUBJUBC;

const isOnCurve = (p) => {
  const { JUBJUBA: a, JUBJUBD: d } = BABYJUBJUB;
  const uu = (p[0] * p[0]) % Fp;
  const vv = (p[1] * p[1]) % Fp;
  const uuvv = (uu * vv) % Fp;
  return (a * uu + vv) % Fp === (one + d * uuvv) % Fp;
}

const edwardsCompress = (p) => {
  const px = utils.decToBin(p[0]);
  const py = utils.decToBin(p[1]);
  const xBits = px.toString(2).padStart(256, '0');
  const yBits = py.toString(2).padStart(256, '0');
  const sign = xBits[255] === '1' ? '1' : '0';
  const yBitsC = sign.concat(yBits.slice(1)); // add in the sign bit
  const y = utils.ensure0x(
    BigInt('0b'.concat(yBitsC))
      .toString(16)
      .padStart(64, '0'),
  ); // put yBits into hex
  return y;
}

const edwardsDecompress = (y) => {
  const { JUBJUBA: a, JUBJUBD: d } = BABYJUBJUB;
  const py = BigInt('0x'.concat(y))
    .toString(2)
    .padStart(256, '0');
  const sign = py[0];
  const yfield = BigInt(`0b${py.slice(1)}`); // remove the sign encoding
  if (yfield > ZOKRATES_PRIME || yfield < 0)
    throw new Error(`y cordinate ${yfield} is not a field element`);
  // 168700.x^2 + y^2 = 1 + 168696.x^2.y^2
  const y2 = mulMod([yfield, yfield]);
  const x2 = modDivide(
    addMod([y2, BigInt(-1)]),
    addMod([mulMod([d, y2]), -a]),
    ZOKRATES_PRIME,
  );
  let xfield = squareRootModPrime(x2);
  const px = BigInt(xfield)
    .toString(2)
    .padStart(256, '0');
  if (px[255] !== sign) xfield = ZOKRATES_PRIME - xfield;
  const p = [xfield, yfield];
  if (!isOnCurve(p)) throw new Error('The computed point was not on the Babyjubjub curve');
  return p;
}

module.exports = {
  isOnCurve,
  edwardsCompress,
  edwardsDecompress,
}
