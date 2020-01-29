const core = require('@actions/core');
const github = require('@actions/github');
const exec = require('@actions/exec');
var fs = require('fs');

try {
  // `who-to-greet` input defined in action metadata file
  const nameToGreet = core.getInput('who-to-greet');
  console.log(__dirname);
  console.log(process.env.HOME);
  const payload = JSON.stringify(github.context.payload, undefined, 2);
  //console.log(`The event payload: ${payload}`);

  fs.writeFile(process.env.HOME + '/.gcp.json', core.getInput('cloud_sdk_service_account_key'), function (err) {
    if (err) throw err;
    console.log('File is created successfully.');
  }); 

  //exec.exec('echo "' + core.getInput('cloud_sdk_service_account_key') + '" > ' + process.env.HOME + '/.gcp.json');
  //exec.exec('echo "foo"');
  //exec.exec('node -version');
  //echo "$CLOUD_SDK_SERVICE_ACCOUNT_KEY" > .gcp.json
  //await exec.exec('gcloud auth activate-service-account --key-file .gcp.json');
} catch (error) {
  core.setFailed(error.message);
}