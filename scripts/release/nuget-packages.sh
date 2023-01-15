set -e
project_dir=$(pwd)
function push() {
  project_file=$1
  sed -i.back "s/999.999.999/$VERSION/g" $project_file
  dir=$(dirname $project_file)
  cd $dir
  dotnet pack --configuration Release
  nupkg_file=$(ls bin/Release/*.nupkg)
  dotnet nuget push $nupkg_file -s https://api.nuget.org/v3/index.json -k $NUGET_PUSH_KEY
  cd $project_dir
  mv $project_file.back $project_file
}

push client/Net/Embedded/Src/Src.csproj
push client/Net/StatusReceiver/Proto/Proto.csproj
