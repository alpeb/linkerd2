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
    await exec.exec('target/cli/linux/linkerd', ['--client', '--short'], {
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
    console.log('Installed Linkerd CLI version', tag)

    // last part is to distinguish runs on the same sha (run-id is unique per CI run)
    var name = `testing-${tag}-${core.getInput('run-id')}`;
    console.log('name', name);
    core.setOutput('name', name);
}

try {
    console.log('run_id', github.context.run_id);
    run()
} catch (error) {
    core.setFailed(error.message);
}