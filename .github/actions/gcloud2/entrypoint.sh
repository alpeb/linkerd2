#!/bin/bash

set -eu

# Install gcloud and kubectl.
echo "$INPUT_CLOUD_SDK_SERVICE_ACCOUNT_KEY" > .gcp.json
dir="${CLOUDSDK_INSTALL_DIR:-${HOME}}/google-cloud-sdk"
(
    . bin/_gcp.sh ;
    install_gcloud "$dir"
    gcloud components install kubectl
    set_gcloud_config "$INPUT_GCP_PROJECT" "$INPUT_GCP_ZONE" foobar
)
. "$dir/path.bash.inc"
gcloud auth configure-docker
echo "::add-path::/home/runner/work/_temp/_github_home/google-cloud-sdk/bin"
gcloud config list
