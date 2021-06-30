#!/bin/bash

project_dir=$(pwd)/../../../

docker run --rm -v$project_dir:/tmp/source -w /tmp/source/proto/auth/cerberus/ akviring/protoc:latest \
  protoc \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=/tmp/source/clients/Unity/Assets/Scripts/Cerberus/grpc/ \
  --csharp_out=/tmp/source/clients/Unity/Assets/Scripts/Cerberus/grpc/ internal.proto

docker run --rm -v$project_dir:/tmp/source -w /tmp/source/proto/auth/cerberus akviring/protoc:latest \
  protoc \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=/tmp/source/clients/Unity/Packages/games.cheetah.cerberus/Runtime/GRPC/ \
  --csharp_out=/tmp/source/clients/Unity/Packages/games.cheetah.cerberus/Runtime/GRPC/ external.proto

docker run --rm -v$project_dir:/tmp/source -w /tmp/source/proto/auth/cerberus akviring/protoc:latest \
  protoc \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=/tmp/source/clients/Unity/Packages/games.cheetah.cerberus/Runtime/GRPC/ \
  --csharp_out=/tmp/source/clients/Unity/Packages/games.cheetah.cerberus/Runtime/GRPC/ types.proto
