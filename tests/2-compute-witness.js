const { exec } = require("child_process");
const fs = require('fs');
const path = require('path');

(async function () {
  // Read witness from ./build/commitment.json
  const fileLocation = './build/commitment.json';
  const parentDir = path.resolve(__dirname, '..');
  const outputFile = path.resolve(parentDir, fileLocation);

  const output = JSON.parse(fs.readFileSync(outputFile, { encoding: 'utf8' }));

  exec("cd build; zokrates compute-witness -a " + output.witnessInputs, (error, stdout, stderr) => {
    if (error) {
      console.log(`error: ${error.message}`);
      return;
    }
    if (stderr) {
      console.log(`stderr: ${stderr}`);
      return;
    }
    console.log(`stdout: ${stdout}`);
  });

})();