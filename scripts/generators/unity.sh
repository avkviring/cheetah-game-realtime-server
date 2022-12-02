#!/bin/bash

source scripts/generators/common.sh
source scripts/common/macos.sh

generate_unity_grpc_files() {
  protofile=$1
  protodir=$(dirname $protofile)
  grpc_out_path=$2
  echo "Creating proto implementation files ($protofile)"
  run_protoc $protofile $protodir $grpc_out_path
}

generate_meta_files() {
  dir=$1
  echo "Creating Unity .meta files $dir"
  for f in $dir/*.cs; do
    guid=$(uuidgen --md5 -n @url -N $f | tr -d '-')
    meta=$f.meta
    echo "fileFormatVersion: 2
guid: $guid
MonoImporter:
externalObjects: {}
serializedVersion: 2
defaultReferences: []
executionOrder: 0
icon: {instanceID: 0}
userData:
assetBundleName:
assetBundleVariant:
" > $meta
  done
}
