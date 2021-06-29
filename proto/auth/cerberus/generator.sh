#!/bin/bash
docker run --rm -v$(pwd)/../../:/tmp/source -w /tmp/source/proto/cerberus/ akviring/protoc:latest \
  protoc \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=../../clients/Unity/Assets/Scripts/Cerberus/grpc/ \
  --csharp_out=../../clients/Unity/Assets/Scripts/Cerberus/grpc/ cerberus.internal.proto

docker run --rm -v$(pwd)/../../:/tmp/source -w /tmp/source/proto/cerberus akviring/protoc:latest \
  protoc \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=../../clients/Unity/Packages/games.cheetah.cerberus/Runtime/GRPC/ \
  --csharp_out=../../clients/Unity/Packages/games.cheetah.cerberus/Runtime/GRPC/ cerberus.external.proto

docker run --rm -v$(pwd)/../../:/tmp/source -w /tmp/source/proto/cerberus akviring/protoc:latest \
  protoc \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=../../clients/Unity/Packages/games.cheetah.cerberus/Runtime/GRPC/ \
  --csharp_out=../../clients/Unity/Packages/games.cheetah.cerberus/Runtime/GRPC/ cerberus.types.proto
