#!/bin/bash

set -eu

# Install gcloud and kubectl.
echo "$INPUT_CLOUD_SDK_SERVICE_ACCOUNT_KEY" > .gcp.json
dir="${CLOUDSDK_INSTALL_DIR:-${HOME}}/google-cloud-sdk"
. bin/_gcp.sh ;
install_gcloud "$dir"
gcloud components install kubectl
gcloud auth activate-service-account --key-file .gcp.json
gcloud config set core/project "$INPUT_GCP_PROJECT"
gcloud config set compute/zone "$INPUT_GCP_ZONE"
gcloud auth configure-docker
#echo "::add-path::/home/runner/work/_temp/_github_home/google-cloud-sdk/bin"
