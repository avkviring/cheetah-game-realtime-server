#/bin/bash

#
# Упаковка собранных приложений в docker файлы
# Приложения уже должны быть собраны с x86_64-unknown-linux-musl
#

set -e
tempdir=$(mktemp -d)
cp modules/grpc_health_probe $tempdir/grpc_health_probe
for f in $(find . -type f -name Dockerfile); do
  project=$(echo $f \
    | sed "{s/\/server\/Dockerfile//g; s/\.\///g}" \
    | tr '\/[:upper:]' '-[:lower:]'
  )
  server="cheetah-$project-server"
  echo "Packaging project $project ($f)"

  cp modules/target/x86_64-unknown-linux-musl/release/$server $tempdir
  docker build $tempdir -f $f -t ghcr.io/cheetah-game-platform/platform/${project}:${version}
  docker push ghcr.io/cheetah-game-platform/platform/${project}:${version}
done
