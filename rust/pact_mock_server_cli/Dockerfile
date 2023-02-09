FROM rust:1-alpine3.15 AS builder

# Add our source code.
ADD . /build

RUN apk --no-cache add gcc musl-dev protobuf protobuf-dev
RUN rustup component add rustfmt

# Fix the cargo manifest so it can be built standalone
RUN sed -i -e 's/pact_matching = {\s*version\s*=\s*"\([^"]*\).*/pact_matching = "\1"/' /build/Cargo.toml
RUN sed -i -e 's/pact_mock_server = {\s*version\s*=\s*"\([^"]*\).*/pact_mock_server = "\1"/' /build/Cargo.toml

# Build our application.
RUN cd build && cargo build --release

# Now, we need to build our _real_ Docker container, copying in the executable.
FROM alpine:3.17
RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /build/target/release/pact_mock_server_cli \
    /usr/local/bin/

EXPOSE 8080 9000-9020

ENTRYPOINT ["/usr/local/bin/pact_mock_server_cli"]
CMD ["start", "--base-port", "9000"]
