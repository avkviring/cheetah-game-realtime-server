#!/bin/bash
project_dir=$(pwd)/../
client_dir_from_protodir=../client

generate_unity_grpc_files() {
  suffix=$1
  project_part=$2
  for f in $(find . -type f -name \*$suffix); do
    protodir=$(dirname $f)
    protofile=$(basename $f)
    ## matches.realtime.shared должен генерироваться в отдельный проект, а не в Runtime
    if [[ "$protofile" == "matches.realtime.shared.proto" ]]; then
        continue
    fi
    unity_project=games.cheetah.$(echo $protofile | sed "s/$suffix//")
    grpc_out_path=$protodir/$client_dir_from_protodir/Unity/$unity_project/$project_part/GRPC
    mkdir -p $grpc_out_path
    echo "$protodir ::: $protofile"
    docker run --rm -v$project_dir:/tmp/source -w /tmp/source/modules akviring/protoc:latest \
      protoc \
      --proto_path=$protodir \
      --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
      --grpc_out=$grpc_out_path \
      --csharp_out=$grpc_out_path \
      --experimental_allow_proto3_optional \
      $protofile
  done
}

generate_unity_grpc_files .external.proto Runtime
generate_unity_grpc_files .admin.proto Editor
generate_unity_grpc_files .shared.proto Runtime

## matches.realtime.shared должен генерироваться в отдельный проект, а не в Runtime
docker run --rm -v$project_dir:/tmp/source -w /tmp/source/modules akviring/protoc:latest \
  protoc \
  --proto_path=/tmp/source/modules/matches/Realtime/proto/ \
  --plugin=protoc-gen-grpc=/usr/bin/grpc_csharp_plugin \
  --grpc_out=/tmp/source/modules/matches/Realtime/client/Unity/games.cheetah.matches.realtime.shared/Runtime/GRPC/ \
  --csharp_out=/tmp/source/modules/matches/Realtime/client/Unity/games.cheetah.matches.realtime.shared/Runtime/GRPC/ \
  --experimental_allow_proto3_optional \
  matches.realtime.shared.proto
