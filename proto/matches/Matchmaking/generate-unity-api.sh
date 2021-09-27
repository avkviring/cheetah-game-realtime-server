#!/bin/bash
project_dir=$(pwd)/../../../

docker run --rm -v$project_dir:/tmp/source -w /tmp/source/proto/matches/Matchmaking akviring/protoc:latest \
  protoc \
  --proto_path=../Relay/ \
  --proto_path=. \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=/tmp/source/clients/Unity/Packages/games.cheetah.matches.matchmaking/Runtime/GRPC/ \
  --csharp_out=/tmp/source/clients/Unity/Packages/games.cheetah.matches.matchmaking/Runtime/GRPC/ \
  matches.matchmaking.external.proto