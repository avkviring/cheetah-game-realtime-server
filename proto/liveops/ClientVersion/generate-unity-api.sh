#!/bin/bash
project_dir=$(pwd)/../../../

docker run --rm -v$project_dir:/tmp/source -w /tmp/source/proto/statistics/Events akviring/protoc:latest \
  protoc \
  --proto_path=. \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=/tmp/source/clients/Unity/Packages/games.cheetah.statistics.events/Runtime/GRPC \
  --csharp_out=/tmp/source/clients/Unity/Packages/games.cheetah.statistics.events/Runtime/GRPC \
  statistics.events.external.proto

