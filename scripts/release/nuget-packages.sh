set -e
project_dir=$(pwd)
deploy() {
  type=$1
  project_files=$(find modules -name \*.$type.grpc.csproj)
  for f in $project_files; do
    sed -i.back "s/999.999.999/$VERSION/g" $f
    dir=$(dirname $f)
    project=$(basename $f)
    nupkg=$(echo $project | sed 's/.csproj//g').$VERSION.nupkg
    echo $nupkg
    cd $dir
    dotnet pack --configuration Release
    dotnet nuget push \
      bin/Release/$nupkg \
      -s https://nuget.pkg.github.com/cheetah-game-platform/ \
      -k $GITHUB_TOKEN
    cd $project_dir
  done
}

deploy shared
deploy admin
deploy internal
deploy external
