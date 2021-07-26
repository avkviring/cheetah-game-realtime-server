#!/bin/bash
project_dir=$(pwd)/../../../

docker run --rm -v$project_dir:/tmp/source -w /tmp/source/proto/auth/User/ akviring/protoc:latest \
  protoc \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=/tmp/source/clients/Unity/Assets/Scripts/User/grpc/ \
  --csharp_out=/tmp/source/clients/Unity/Assets/Scripts/User/grpc/ auth.user.internal.proto