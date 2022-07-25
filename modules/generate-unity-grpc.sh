#!/bin/bash

project_dir=$(pwd)/../
client_dir_from_protodir=../client

run_protoc() {
    protofile=$1
    protodir=$2
    grpc_out_path=$3

    mkdir -p $grpc_out_path
    docker run --rm -v$project_dir:/tmp/source -w /tmp/source/modules akviring/protoc:latest \
      protoc \
      --proto_path=$protodir \
      --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
      --grpc_out=$grpc_out_path \
      --csharp_out=$grpc_out_path \
      --experimental_allow_proto3_optional \
      $protofile
}

generate_unity_grpc_files() {
  suffix=$1
  project_part=$2
  echo "Creating proto implementation files ($suffix)"
  for f in $(find . -type f -name \*$suffix); do
    protodir=$(dirname $f)
    protofile=$(basename $f)
    unity_project=games.cheetah.$(echo $protofile | sed "s/$suffix//")
    if [[ $protofile == *.shared.proto ]]; then
      unity_project+=".shared"
    fi
    grpc_out_path=$protodir/$client_dir_from_protodir/Unity/$unity_project/$project_part/GRPC
    echo "  for $protofile in $(realpath --relative-to . $grpc_out_path)..."
    run_protoc $protofile $protodir $grpc_out_path
  done
}

generate_unity_grpc_files .external.proto Runtime
echo
generate_unity_grpc_files .admin.proto Editor
echo
generate_unity_grpc_files .shared.proto Runtime
