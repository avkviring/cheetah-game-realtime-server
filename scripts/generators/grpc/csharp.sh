#!/bin/bash

source scripts/generators/grpc/common.sh

generate_grpc_projects() {
  suffix=$1
  type=$2
  echo "Creating proto implementation files ($suffix)"
  for f in $(find . -type f -name \*$suffix); do
    protodir=$(dirname $f)
    protofile=$(basename $f)

    project=$(echo games.cheetah.$(echo $protofile | sed "s/$suffix//").grpc | tr '[:upper:]' '[:lower:]')
    grpc_out_path=$protodir/$client_dir_from_protodir/csharp/$project

    mkdir -p $grpc_out_path
    echo "  for $protofile in $(realpath --relative-to . $grpc_out_path)"
    run_protoc $protofile $protodir $grpc_out_path
    csproj="
      <Project Sdk='Microsoft.NET.Sdk'>
        <PropertyGroup>
          <TargetFramework>netcoreapp3.1</TargetFramework>
          <Version>999.999.999</Version>
          <PackageId>$project</PackageId>
          <RepositoryUrl>https://github.com/cheetah-game-platform/platform</RepositoryUrl>
        </PropertyGroup>
        <ItemGroup>
          <PackageReference Include='Google.Protobuf' Version='3.21.5' />
          <PackageReference Include='Grpc.Core' Version='2.46.3' />
        </ItemGroup>
      </Project>
  "
    echo $csproj >$grpc_out_path/$project.csproj
  done

}

generate_grpc_projects .external.proto External
echo
generate_grpc_projects .admin.proto Admin
echo
generate_grpc_projects .shared.proto Shared
