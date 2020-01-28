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
    set_gcloud_config "$INPUT_GCP_PROJECT" "$INPUT_GCP_ZONE" "$INPUT_CLUSTER"
    # Get a kubernetes context.
    get_k8s_ctx "$INPUT_GCP_PROJECT" "$INPUT_GCP_ZONE" "$INPUT_CLUSTER"
)
. "$dir/path.bash.inc"
gcloud auth configure-docker
bin/kubectl version --short

# so that it's modifiable by the runner's user
chown -R 777 ~HOME/.config/gcloud