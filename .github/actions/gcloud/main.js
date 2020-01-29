const core = require('@actions/core');
const github = require('@actions/github');
const exec = require('@actions/exec');
var fs = require('fs');

async function configure() {
    await exec.exec(`gcloud auth activate-service-account --key-file ${process.env.HOME}/.gcp.json`);
    await exec.exec(`gcloud config set core/project "${core.getInput('gcp_project')}"`);
    await exec.exec(`gcloud config set compute/zone "${core.getInput('gcp_zone')}"`);
    await exec.exec(`gcloud config set container/cluster "${core.getInput('cluster')}"`);
    await exec.exec(`gcloud container clusters get-credentials "${core.getInput('cluster')}"`);
}

try {
  fs.writeFile(process.env.HOME + '/.gcp.json', core.getInput('cloud_sdk_service_account_key'), function (err) {
      configure()
  }); 
} catch (error) {
    core.setFailed(error.message);
}
