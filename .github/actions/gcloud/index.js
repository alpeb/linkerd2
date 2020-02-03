const core = require('@actions/core');
const exec = require('@actions/exec');
const fs = require('fs');

async function getClusterName() {
  let tag, clientVersion;
  await exec.exec('bin/root-tag', [], {
      env: {
          CI_FORCE_CLEAN: 1
      },
      listeners: {
          stdout: (data) => {
              tag = data.toString().trim()
          }
      }
  });

  await exec.exec(`${process.env.HOME}/.linkerd version --client --short`, [], {
      listeners: {
          stdout: (data) => {
              clientVersion = data.toString().trim()
          }
      }
  });

  console.log('here');
  // validate CLI version matches the repo
  if (tag !== clientVersion) {
    console.log("bad tag");
      throw `tag ${tag} differs from client version ${clientVersion}`
  }
  console.log('Linkerd CLI version:', tag)

  // Last part is to distinguish runs on the same sha (run_id is unique per CI run).
  // run_id has to be provided as an input because it turns out it's not available
  // through github.context.run_id
  const name = `testing-${tag}-${core.getInput('run_id')}`;
  console.log('Cluster name:', name);
  return name;
}

function validate() {
  switch (core.getInput('action')) {
    case 'create':
      break;
    case 'destroy':
      break;
    case '':
      break;
    default:
      throw 'Invalid value for "action"';
  }
}

async function configure() {
  try {
    await exec.exec('gcloud auth activate-service-account',
      ['--key-file',  `${process.env.HOME}/.gcp.json`]);
    await exec.exec('gcloud config set core/project', [core.getInput('gcp_project')]);
    await exec.exec('gcloud config set compute/zone', [core.getInput('gcp_zone')]);
    await exec.exec('gcloud auth configure-docker --quiet');

    if (core.getInput('create') || core.getInput('destroy')) {
      const name = await getClusterName();
      if (core.getInput('create')) {
        const args = [
          name,
          '--machine-type', core.getInput('machine_type'),
          '--num-nodes', core.getInput('num_nodes'),
          '--cluster-version', core.getInput('cluster_version'),
          '--release-channel', core.getInput('release_channel')
        ];
        if (core.getInput('preemptible')) {
          args.push('--preemptible');
        }
        if (core.getInput('enable_network_policy')) {
          args.push('--enable-network-policy')
        }
        if (!core.getInput('enable_stackdriver')) {
          args.push('--no-enable-stackdriver-kubernetes')
        }
        if (!core.getInput('enable_basic_auth')) {
          args.push('--no-enable-basic-auth')
        }
        if (!core.getInput('enable_legacy_auth')) {
          args.push('--no-enable-legacy-authorization')
        }
        await exec.exec('gcloud container clusters create', args);

        await exec.exec('gcloud config set container/cluster',  [name]);
        await exec.exec('gcloud container clusters get-credentials', [name]);

        let sa;
        await exec.exec('gcloud config get-value account', [], {
          listeners: {
            stdout: (data) => {
              sa = data.toString()
            }
          }
        });
        await exec.exec('kubectl create clusterrolebinding ci-cluster-admin --clusterrole=cluster-admin',
          ['--user', sa]);
      } else {
        await exec.exec('gcloud container clusters delete --quiet', [name]);
      }
    }
  } catch (e) {
    console.log("err0: ", e.message);
    core.setFailed(e.message)
  }
}

try {
    fs.writeFileSync(process.env.HOME + '/.gcp.json', core.getInput('cloud_sdk_service_account_key'));
    validate();
    configure();
} catch (e) {
  console.log("err1: ", e.message);
    core.setFailed(e.message);
}
