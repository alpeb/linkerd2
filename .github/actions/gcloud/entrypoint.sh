#!/bin/bash

set -eu

# Install gcloud and kubectl.
echo "$INPUT_CLOUD_SDK_SERVICE_ACCOUNT_KEY" > .gcp.json
dir="${CLOUDSDK_INSTALL_DIR:-${HOME}}/google-cloud-sdk"
(
    . bin/_gcp.sh ;
    install_gcloud "$dir"
    gcloud components install kubectl
    # Configure gcloud with a service account.
    set_gcloud_config "$GCP_PROJECT" "$GCP_ZONE" "$GKE_CLUSTER"
    # Get a kubernetes context.
    get_k8s_ctx "$GCP_PROJECT" "$GCP_ZONE" "$GKE_CLUSTER"
)
. "$dir/path.bash.inc"
gcloud auth configure-docker
bin/kubectl version --short