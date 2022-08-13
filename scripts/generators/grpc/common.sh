#!/bin/bash

project_dir=$(pwd)
client_dir_from_protodir=../client

run_protoc() {
  protofile=$1
  protodir=$2
  grpc_out_path=$3
  mkdir -p $grpc_out_path
  docker run --rm -v$project_dir:/tmp/source -w /tmp/source akviring/protoc:latest \
    protoc \
    --proto_path=$protodir \
    --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
    --grpc_out=$grpc_out_path \
    --csharp_out=$grpc_out_path \
    --experimental_allow_proto3_optional \
    $protofile
}
