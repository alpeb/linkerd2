#!/bin/bash

set -eu

# Install gcloud and kubectl.
echo "$INPUT_CLOUD_SDK_SERVICE_ACCOUNT_KEY" > .gcp.json
dir="${CLOUDSDK_INSTALL_DIR:-${HOME}}/google-cloud-sdk"
(
    . bin/_gcp.sh ;
    install_gcloud "$dir"
    gcloud components install kubectl
)
. "$dir/path.bash.inc"
gcloud auth configure-docker
echo $GITHUB_WORKSPACE
//echo "::add-path::/path/to/dir"
echo $PATH
gcloud version
