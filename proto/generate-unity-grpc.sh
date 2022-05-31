#!/bin/bash
project_dir=$(pwd)/../

for f in $(find . -name "*.external.proto"); do
  dir=$(dirname $f)
  file=$(basename $f)
  echo "$dir ::: $file"
  unity_project=games.cheetah.$(echo $file | sed 's/.external.proto//g')
  docker run --rm -v$project_dir:/tmp/source -w /tmp/source/proto/$dir akviring/protoc:latest \
    protoc \
    --proto_path=. \
    --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
    --grpc_out=/tmp/source/clients/Unity/Packages/$unity_project/Runtime/GRPC \
    --csharp_out=/tmp/source/clients/Unity/Packages/$unity_project/Runtime/GRPC \
    $file
done

for f in $(find . -name "*.admin.proto"); do
  dir=$(dirname $f)
  file=$(basename $f)
  echo "$dir ::: $file"
  unity_project=games.cheetah.$(echo $file | sed 's/.admin.proto//g')
  docker run --rm -v$project_dir:/tmp/source -w /tmp/source/proto/$dir akviring/protoc:latest \
    protoc \
    --proto_path=. \
    --experimental_allow_proto3_optional \
    --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
    --grpc_out=/tmp/source/clients/Unity/Packages/$unity_project/Editor/GRPC \
    --csharp_out=/tmp/source/clients/Unity/Packages/$unity_project/Editor/GRPC \
    $file
done

docker run --rm -v$project_dir:/tmp/source -w /tmp/source/proto/matches/Relay akviring/protoc:latest \
  protoc \
  --proto_path=. \
  --experimental_allow_proto3_optional \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=/tmp/source/clients/Unity/Packages/games.cheetah.matches.relay/Editor/GRPC \
  --csharp_out=/tmp/source/clients/Unity/Packages/games.cheetah.matches.relay/Editor/GRPC \
  matches.relay.admin.proto
