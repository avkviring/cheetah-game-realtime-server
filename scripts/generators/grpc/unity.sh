#!/bin/bash

source scripts/generators/grpc/common.sh
source scripts/common/macos.sh

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

generate_meta_files() {
  generated_scripts=$(find . -type f -name '*.cs' | grep '.*/Unity/.*\(Editor\|Runtime\)/GRPC.*.cs')
  echo 'Creating Unity .meta files'
  for f in $generated_scripts; do
    project_part=$(basename $(dirname $(dirname $f)))
    file_name=$(basename $f)
    guid=$(uuidgen --md5 -n @url -N Unity/$project_part/$file_name | tr -d '-')
    echo "  for $(basename $f) (guid: $guid)..."
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
" >$f.meta

  done
}

generate_unity_grpc_files .external.proto Runtime
echo
generate_unity_grpc_files .admin.proto Editor
echo
generate_unity_grpc_files .shared.proto Runtime

# мета файлы создаем только для linux - так как
# именно он используется для вывода релиза
if [[ ($OSTYPE == 'linux'*)  ]]; then
generate_meta_files
fi