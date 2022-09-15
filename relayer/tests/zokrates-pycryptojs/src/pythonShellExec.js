let PythonShell = require('python-shell');

const pythonExec = (pythonScript, args) => {
  return new Promise((done, reject) => {
    PythonShell.PythonShell.run(pythonScript, { args, scriptPath: __dirname }, (err, results) =>
      err ? reject(err) : done(results),
    );
  });
};

const pyScriptExec = async (scriptFilePath, execArgs) => {
  let execOut;
  try {
    execOut = await pythonExec(scriptFilePath, execArgs);
    execOut = execOut[0].split(' ');
  } catch (err) {
    console.log(err);
    process.exit(1);
  }
  return execOut;
};

module.exports = {
  pyScriptExec,
}
