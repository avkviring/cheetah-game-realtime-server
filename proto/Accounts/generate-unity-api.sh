#!/bin/bash
project_dir=$(pwd)/../../
docker run --rm -v$project_dir:/tmp/source -w /tmp/source/proto/Accounts akviring/protoc:latest \
  protoc \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=/tmp/source/clients/Unity/Packages/games.cheetah.accounts/Runtime/GRPC/ \
  --csharp_out=/tmp/source/clients/Unity/Packages/games.cheetah.accounts/Runtime/GRPC/ accounts.external.proto

