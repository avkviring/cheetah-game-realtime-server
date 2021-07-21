#!/bin/bash
project_dir=$(pwd)/../../../

docker run --rm -v$project_dir:/tmp/source -w /tmp/source/proto/auth/Cerberus/ akviring/protoc:latest \
  protoc \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=/tmp/source/clients/Unity/Assets/Scripts/Cerberus/grpc/ \
  --csharp_out=/tmp/source/clients/Unity/Assets/Scripts/Cerberus/grpc/ auth.cerberus.internal.proto

docker run --rm -v$project_dir:/tmp/source -w /tmp/source/proto/auth/Cerberus akviring/protoc:latest \
  protoc \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=/tmp/source/clients/Unity/Packages/games.cheetah.auth.cerberus/Runtime/GRPC/ \
  --csharp_out=/tmp/source/clients/Unity/Packages/games.cheetah.auth.cerberus/Runtime/GRPC/ auth.cerberus.external.proto

docker run --rm -v$project_dir:/tmp/source -w /tmp/source/proto/auth/Cerberus akviring/protoc:latest \
  protoc \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=/tmp/source/clients/Unity/Packages/games.cheetah.auth.cerberus/Runtime/GRPC/ \
  --csharp_out=/tmp/source/clients/Unity/Packages/games.cheetah.auth.cerberus/Runtime/GRPC/ auth.cerberus.types.proto
