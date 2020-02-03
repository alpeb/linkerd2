module.exports =
/******/ (function(modules, runtime) { // webpackBootstrap
/******/ 	"use strict";
/******/ 	// The module cache
/******/ 	var installedModules = {};
/******/
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/
/******/ 		// Check if module is in cache
/******/ 		if(installedModules[moduleId]) {
/******/ 			return installedModules[moduleId].exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = installedModules[moduleId] = {
/******/ 			i: moduleId,
/******/ 			l: false,
/******/ 			exports: {}
/******/ 		};
/******/
/******/ 		// Execute the module function
/******/ 		modules[moduleId].call(module.exports, module, module.exports, __webpack_require__);
/******/
/******/ 		// Flag the module as loaded
/******/ 		module.l = true;
/******/
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/
/******/
/******/ 	__webpack_require__.ab = __dirname + "/";
/******/
/******/ 	// the startup function
/******/ 	function startup() {
/******/ 		// Load entry module and return exports
/******/ 		return __webpack_require__(266);
/******/ 	};
/******/
/******/ 	// run startup
/******/ 	return startup();
/******/ })
/************************************************************************/
/******/ ({

/***/ 266:
/***/ (function(__unusedmodule, __unusedexports, __webpack_require__) {

const core = __webpack_require__(525);
const exec = __webpack_require__(267);
const fs = __webpack_require__(747);

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

  // validate CLI version matches the repo
  /*if (tag !== clientVersion) {
      throw `tag ${tag} differs from client version ${clientVersion}`
  }*/
  tag="master"
  console.log('Linkerd CLI version:', tag)

  // Last part is to distinguish runs on the same sha (run_id is unique per CI run).
  // run_id has to be provided as an input because it turns out it's not available
  // through github.context.run_id
  const name = `testing-${tag}-${core.getInput('run_id')}`;
  console.log('Cluster name:', name);
  return name;
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
    core.setFailed(e.message)
  }
}

try {
    fs.writeFileSync(process.env.HOME + '/.gcp.json', core.getInput('cloud_sdk_service_account_key'));
    validate();
    configure();
} catch (e) {
    core.setFailed(e.message);
}


/***/ }),

/***/ 267:
/***/ (function() {

eval("require")("@actions/exec");


/***/ }),

/***/ 525:
/***/ (function() {

eval("require")("@actions/core");


/***/ }),

/***/ 747:
/***/ (function(module) {

module.exports = require("fs");

/***/ })

/******/ });