#!/bin/bash
docker run --rm -v$(pwd)/../../:/tmp/source -w /tmp/source/proto/auth/ akviring/protoc:latest \
  protoc \
  --proto_path=../cerberus/ \
  --proto_path=. \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=../../clients/Unity/Packages/games.cheetah.auth.cookie/Runtime/grpc/  \
  --csharp_out=../../clients/Unity/Packages/games.cheetah.auth.cookie/Runtime/grpc/ cookie.proto \

docker run --rm -v$(pwd)/../../:/tmp/source -w /tmp/source/proto/auth akviring/protoc:latest \
  protoc \
  --proto_path=../cerberus/ \
  --proto_path=. \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=../../clients/Unity/Packages/games.cheetah.auth.android/Runtime/grpc/  \
  --csharp_out=../../clients/Unity/Packages/games.cheetah.auth.android/Runtime/grpc/ google.proto \

