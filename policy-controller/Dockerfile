ARG RUST_IMAGE=docker.io/library/rust:1.60.0-buster
ARG RUNTIME_IMAGE=gcr.io/distroless/cc

# Builds the controller binary.
FROM $RUST_IMAGE as build
ARG TARGETARCH
WORKDIR /build
COPY bin/scurl bin/scurl
COPY Cargo.toml Cargo.lock .
COPY policy-controller policy-controller
RUN cargo new policy-test --lib
RUN target=$(rustup show | sed -n 's/^Default host: \(.*\)/\1/p'); \
    --mount=type=cache,target=target \
    --mount=type=cache,from=rust:1.60.0-buster,source=/usr/local/cargo,target=/usr/local/cargo \
    echo "TARGET: $target" && cargo fetch --target ${target}
RUN target=$(rustup show | sed -n 's/^Default host: \(.*\)/\1/p'); \
    --mount=type=cache,target=target \
    --mount=type=cache,from=rust:1.60.0-buster,source=/usr/local/cargo,target=/usr/local/cargo \
    cargo build --frozen --target="$target" --release --package=linkerd-policy-controller && \
    mv "target/${target}/release/linkerd-policy-controller" /tmp/
# Creates a minimal runtime image with the controller binary.
FROM $RUNTIME_IMAGE
COPY --from=build /tmp/linkerd-policy-controller /bin/
ENTRYPOINT ["/bin/linkerd-policy-controller"]
