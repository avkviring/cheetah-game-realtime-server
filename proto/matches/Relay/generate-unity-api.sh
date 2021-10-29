#!/bin/bash
project_dir=$(pwd)/../../../

docker run --rm -v$project_dir:/tmp/source -w /tmp/source/proto/matches/Relay akviring/protoc:latest \
  protoc \
  --proto_path=. \
  --experimental_allow_proto3_optional \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=/tmp/source/clients/Unity/Packages/games.cheetah.matches.relay/Editor/GRPC \
  --csharp_out=/tmp/source/clients/Unity/Packages/games.cheetah.matches.relay/Editor/GRPC \
  matches.relay.admin.proto

  docker run --rm -v$project_dir:/tmp/source -w /tmp/source/proto/matches/Relay akviring/protoc:latest \
    protoc \
    --proto_path=. \
    --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
    --grpc_out=/tmp/source/clients/Unity/Packages/games.cheetah.matches.relay.shared/Runtime/GRPC \
    --csharp_out=/tmp/source/clients/Unity/Packages/games.cheetah.matches.relay.shared/Runtime/GRPC \
    matches.relay.shared.proto