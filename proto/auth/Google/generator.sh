#!/bin/bash
project_dir=$(pwd)/../../../

docker run --rm -v$project_dir:/tmp/source -w /tmp/source/proto/auth/Google akviring/protoc:latest \
  protoc \
  --proto_path=../Cerberus/ \
  --proto_path=. \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=/tmp/source/clients/Unity/Packages/games.cheetah.authentication.android/Runtime/grpc/ \
  --csharp_out=/tmp/source/clients/Unity/Packages/games.cheetah.authentication.android/Runtime/grpc/ \
  auth.google.external.proto
