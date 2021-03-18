#!/bin/bash
docker run -v$(pwd)/../:/tmp/source -w /tmp/source/proto akviring/protoc:latest \
    protoc \
    --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
    --grpc_out=../clients/Unity/Assets/Tests/ \
    --csharp_out=../clients/Unity/Assets/Tests/ cerberus.internal.proto

docker run -v$(pwd)/../:/tmp/source -w /tmp/source/proto akviring/protoc:latest \
  protoc \
    --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
    --grpc_out=../clients/Unity/Packages/games.cheetah.unity.cerberus/Runtime/grpc/ \
    --csharp_out=../clients/Unity/Packages/games.cheetah.unity.cerberus/Runtime/grpc/ cerberus.external.proto

docker run -v$(pwd)/../:/tmp/source -w /tmp/source/proto akviring/protoc:latest \
  protoc \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=../clients/Unity/Packages/games.cheetah.unity.cerberus/Runtime/grpc/ \
  --csharp_out=../clients/Unity/Packages/games.cheetah.unity.cerberus/Runtime/grpc/ cerberus.types.proto
