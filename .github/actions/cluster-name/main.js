const core = require('@actions/core');
const exec = require('@actions/exec');
const github = require('@actions/github');

async function run() {
    var tag;
    await exec.exec('bin/root-tag', [], {
        env: {
            CI_FORCE_CLEAN: 1
        },
        listeners: {
            stdout: (data) => {
                tag = data.toString()
            }
        }
    });

    var clientVersion;
    await exec.exec('target/cli/linux/linkerd', ['version', '--client', '--short'], {
        listeners: {
            stdout: (data) => {
                clientVersion = data.toString()
            }
        }
    });

    // validate CLI version matches the repo
    if (tag != clientVersion) {
        core.setFailed(`tag ${tag} differs from clientversion ${clientVersion}`)
    }
    console.log('Installed Linkerd CLI version:', tag)

    // Last part is to distinguish runs on the same sha (run-id is unique per CI run).
    // run-id has to be provided as an input because it turns out it's not available
    // through github.context.run_id
    var name = `testing-${tag}-${core.getInput('run-id')}`;
    console.log('name:', name);
    core.setOutput('name', name);
}

try {
    run()
} catch (error) {
    core.setFailed(error.message);
}