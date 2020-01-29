const core = require('@actions/core');
const exec = require('@actions/exec');
var fs = require('fs');

async function configure() {
    await exec.exec(`gcloud auth activate-service-account --key-file ${process.env.HOME}/.gcp.json`);
    await exec.exec(`gcloud config set core/project "${core.getInput('gcp_project')}"`);
    await exec.exec(`gcloud config set compute/zone "${core.getInput('gcp_zone')}"`);
    await exec.exec(`gcloud config set container/cluster "${core.getInput('cluster')}"`);
    await exec.exec(`gcloud container clusters get-credentials "${core.getInput('cluster')}"`);
    await exec.exec('gcloud auth configure-docker --quiet');
    await exec.exec('gcloud config get-value account', [], {
      listeners: {
        stdout: (data) => {
          console.log("DATA: ", data.toString())
        }
      }
    });
    await exec.exec(`kubectl create clusterrolebinding ci-cluster-admin --clusterrole=cluster-admin --user=test-ci-eraseme@linkerd-io.iam.gserviceaccount.com`);
}

try {
  fs.writeFile(process.env.HOME + '/.gcp.json', core.getInput('cloud_sdk_service_account_key'), function (err) {
      configure()
  }); 
} catch (error) {
    core.setFailed(error.message);
}
