FROM rust:1-alpine3.16 AS builder

# Add our source code.
ADD . /build

RUN apk --no-cache add gcc musl-dev protobuf protobuf-dev
RUN rustup component add rustfmt

# Fix the cargo manifest so it can be built standalone
RUN sed -i -e 's/pact_verifier = {\s*version\s*=\s*"\([^"]*\).*/pact_verifier = "\1"/' /build/Cargo.toml

# Build our application.
RUN cd build && cargo build --release

# Now, we need to build our _real_ Docker container, copying in the executable.
FROM alpine:3
RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /build/target/release/pact_verifier_cli \
    /usr/local/bin/

ENTRYPOINT ["/usr/local/bin/pact_verifier_cli"]
