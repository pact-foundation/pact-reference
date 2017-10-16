#Copyright: Amdocs Development Limited, 2017
FROM scratch

LABEL maintainer="assaf.katz@amdocs.com"

ADD target/x86_64-unknown-linux-musl/release/CLI_NAME /  
EXPOSE 8080

ENTRYPOINT ["/CLI_NAME"]  
