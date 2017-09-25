FROM scratch

ADD target/x86_64-unknown-linux-musl/release/pact_verifier_cli /  
EXPOSE 8080

ENTRYPOINT ["/pact_verifier_cli"]  
