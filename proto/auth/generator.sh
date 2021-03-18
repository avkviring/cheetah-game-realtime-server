#!/bin/bash
docker run -v$(pwd)/../../:/tmp/source -w /tmp/source/auth/proto akviring/protoc:latest \
  protoc \
  --proto_path=../../cerberus/proto/ \
  --proto_path=. \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=../clients/Unity/Packages/games.cheetah.unity.auth.cookie/Runtime/grpc/  \
  --csharp_out=../clients/Unity/Packages/games.cheetah.unity.auth.cookie/Runtime/grpc/ cookie.proto \

docker run -v$(pwd)/../../:/tmp/source -w /tmp/source/auth/proto akviring/protoc:latest \
  protoc \
  --proto_path=../../cerberus/proto/ \
  --proto_path=. \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=../clients/Unity/Packages/games.cheetah.unity.auth.android/Runtime/grpc/  \
  --csharp_out=../clients/Unity/Packages/games.cheetah.unity.auth.android/Runtime/grpc/ google.proto \

