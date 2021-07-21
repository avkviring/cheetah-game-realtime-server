#!/bin/bash
project_dir=$(pwd)/../../../

docker run --rm -v$project_dir:/tmp/source -w /tmp/source/proto/auth/Cookie akviring/protoc:latest \
  protoc \
  --proto_path=../Cerberus/ \
  --proto_path=. \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=/tmp/source/clients/Unity/Packages/games.cheetah.auth.cookie/Runtime/GRPC/  \
  --csharp_out=/tmp/source/clients/Unity/Packages/games.cheetah.auth.cookie/Runtime/GRPC/ \
  auth.cookie.external.proto