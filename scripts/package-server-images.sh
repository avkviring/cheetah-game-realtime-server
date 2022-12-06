#/bin/bash

#
# Упаковка собранных приложений в docker файлы
# Приложения уже должны быть собраны с x86_64-unknown-linux-musl
#

set -e
tempdir=$(mktemp -d)
cp rust/grpc_health_probe $tempdir/grpc_health_probe
cp -R rust/target/x86_64-unknown-linux-musl/release/ $tempdir
docker build $tempdir -f rust/Server/Dockerfile -t ghcr.io/cheetah-game-platform/platform/cheetah-server:${version}
docker build $tempdir -f rust/Registry/Dockerfile -t ghcr.io/cheetah-game-platform/platform/cheetah-registry:${version}

docker push ghcr.io/cheetah-game-platform/platform/cheetah-server:${version}
docker push ghcr.io/cheetah-game-platform/platform/cheetah-registry:${version}
