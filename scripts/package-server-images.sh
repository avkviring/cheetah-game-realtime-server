#!/bin/bash

#
# Упаковка собранных приложений в docker файлы
# Приложения уже должны быть собраны с x86_64-unknown-linux-musl
#

set -e
tempdir=$(mktemp -d)
cp rust/grpc_health_probe $tempdir/grpc_health_probe
cp -R rust/target/x86_64-unknown-linux-musl/release/cheetah-server $tempdir/cheetah-server
docker build $tempdir -f rust/Server/Dockerfile -t ghcr.io/avkviring/platform/cheetah-server:${version}
docker push ghcr.io/avkviring/platform/cheetah-server:${version}
