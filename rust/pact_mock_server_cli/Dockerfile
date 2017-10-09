#Copyright: Amdocs Development Limited, 2017
FROM scratch

LABEL maintainer="assaf.katz@amdocs.com"

ADD target/x86_64-unknown-linux-musl/release/pact_mock_server_cli /  
EXPOSE 8080

ENTRYPOINT ["/pact_mock_server_cli"]  
