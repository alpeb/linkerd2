const core = require('@actions/core');
const exec = require('@actions/exec');
var fs = require('fs');

async function configure() {
    await exec.exec(`gcloud auth activate-service-account --key-file ${process.env.HOME}/.gcp.json`);
    await exec.exec(`gcloud config set core/project "${core.getInput('gcp_project')}"`);
    await exec.exec(`gcloud config set compute/zone "${core.getInput('gcp_zone')}"`);
    await exec.exec('gcloud auth configure-docker --quiet');

    var cluster = core.getInput('cluster')
    if (cluster) {
      await exec.exec(`gcloud container clusters create ${cluster} --cluster-version 1.15.7-gke.23 --num-nodes=1 --machine-type n1-standard-2 --enable-network-policy`);
      await exec.exec(`gcloud config set container/cluster "${cluster}"`);
      await exec.exec(`gcloud container clusters get-credentials "${cluster}"`);
      var sa;
      await exec.exec('gcloud config get-value account', [], {
        listeners: {
          stdout: (data) => {
            sa = data.toString()
          }
        }
      });
      await exec.exec(`kubectl create clusterrolebinding ci-cluster-admin --clusterrole=cluster-admin --user=${sa}`);
    }
}

try {
  fs.writeFile(process.env.HOME + '/.gcp.json', core.getInput('cloud_sdk_service_account_key'), function (err) {
      configure()
  }); 
} catch (error) {
    core.setFailed(error.message);
}
