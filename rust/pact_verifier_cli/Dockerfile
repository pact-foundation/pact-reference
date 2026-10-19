# Packages the static musl pact-verifier from a dist release. No step runs
# on the target platform, so a multi-platform build needs no emulation.
FROM --platform=$BUILDPLATFORM alpine:3.22 AS fetch
ARG VERSION
ARG TARGETARCH
RUN apk add --no-cache ca-certificates curl xz
RUN case "$TARGETARCH" in \
        amd64) target=x86_64-unknown-linux-musl ;; \
        arm64) target=aarch64-unknown-linux-musl ;; \
        *) echo "unsupported architecture: $TARGETARCH" >&2; exit 1 ;; \
    esac \
    && curl -fsSL "https://github.com/pact-foundation/pact-reference/releases/download/pact_verifier_cli-v${VERSION}/pact_verifier_cli-${target}.tar.xz" \
       | tar xJ --strip-components=1 \
    && install -m 0755 pact-verifier /pact-verifier

FROM alpine:3.22
COPY --from=fetch /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=fetch /pact-verifier /usr/local/bin/pact-verifier
ENTRYPOINT ["/usr/local/bin/pact-verifier"]
