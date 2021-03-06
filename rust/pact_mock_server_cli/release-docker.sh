#!/bin/bash

set -e

if [ "" = "$1" ]; then
  echo "Usage: "
  echo "  ./release-docker.sh version"
  exit 1
fi

docker build . -t pactfoundation/pact-ref-mock-server:$1
docker push pactfoundation/pact-ref-mock-server:$1
docker tag pactfoundation/pact-ref-mock-server:$1 pactfoundation/pact-ref-mock-server:latest
docker push pactfoundation/pact-ref-mock-server:latest
