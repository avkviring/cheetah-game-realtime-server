#/bin/bash

#
# Упаковка собранных приложений в docker файлы
# Приложения уже должны быть собраны с x86_64-unknown-linux-musl
#

set -e
rm -rf /tmp/context/
mkdir -p /tmp/context/
cp grpc_health_probe /tmp/context/grpc_health_probe
project_dir=$(pwd)/../
for f in $(find . -type f -name Dockerfile); do
  echo $f
  dir=$(dirname $f)
  project=$(echo $f | sed "s/\/server\/Dockerfile//g" | sed "s/\.\///g" | sed "s/\//-/g" | tr '[:upper:]' '[:lower:]')
  server="cheetah-$project-server"
  cp target/x86_64-unknown-linux-musl/release/$server /tmp/context/
  docker build /tmp/context/ -f $f -t ghcr.io/cheetah-game-platform/platform/${project}:${version}
  docker push ghcr.io/cheetah-game-platform/platform/${project}:${version}
done

#set -e
#bins=()
#bins+=("Accounts/Dockerfile:cheetah-accounts")
#bins+=("matches/Relay/Server/Dockerfile:cheetah-matches-relay")
#bins+=("matches/StubMatchmaking/Dockerfile:cheetah-matches-stub-matchmaking")
#bins+=("matches/StubRegistry/Dockerfile:cheetah-matches-stub-registry")
#bins+=("matches/Factory/Dockerfile:cheetah-matches-factory")
#bins+=("matches/Registry/Dockerfile:cheetah-matches-registry")
#bins+=("statistics/Events/Dockerfile:cheetah-statistics-events")
#bins+=("system/Compatibility/Dockerfile:cheetah-system-compatibility")
#
#
#rm -rf /tmp/context/
#mkdir /tmp/context/
#cp grpc_health_probe /tmp/context/grpc_health_probe
#for bin in ${bins[@]}; do
#  VALUE="${bin##*:}"
#  cp target/x86_64-unknown-linux-musl/release/$VALUE /tmp/context/
#done
#
#for bin in ${bins[@]}; do
#  KEY="${bin%%:*}"
#  VALUE="${bin##*:}"
#  docker build /tmp/context/ -f $KEY -t registry.dev.cheetah.games/cheetah/platform/${VALUE}:${version}
#done
#
##for bin in ${bins[@]}; do
##  VALUE="${bin##*:}"
##  docker push registry.dev.cheetah.games/cheetah/platform/${VALUE}:${version}
##done
