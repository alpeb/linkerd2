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

gcloud auth configure-docker

gcloud version