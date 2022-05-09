#!/bin/bash
project_dir=$(pwd)/../../../

docker run --rm -v$project_dir:/tmp/source -w /tmp/source/proto/system/Compatibility akviring/protoc:latest \
  protoc \
  --proto_path=. \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=/tmp/source/clients/Unity/Packages/games.cheetah.system.compatibility/Runtime/GRPC \
  --csharp_out=/tmp/source/clients/Unity/Packages/games.cheetah.system.compatibility/Runtime/GRPC \
  system.compatibility.external.proto

