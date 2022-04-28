#/bin/bash

#
# Упаковка собранных приложений в docker файлы
# Приложения уже должны быть собраны с x86_64-unknown-linux-musl
#

set -e
bins=()
bins+=("Accounts/Dockerfile:cheetah-accounts")
bins+=("matches/Relay/Server/Dockerfile:cheetah-matches-relay")
bins+=("matches/StubMatchmaking/Dockerfile:cheetah-matches-stub-matchmaking")
bins+=("matches/StubRegistry/Dockerfile:cheetah-matches-stub-registry")
bins+=("matches/Factory/Dockerfile:cheetah-matches-factory")
bins+=("matches/Registry/Dockerfile:cheetah-matches-registry")
bins+=("statistics/Events/Dockerfile:cheetah-statistics-events")

rm -rf /tmp/context/
mkdir /tmp/context/
cp grpc_health_probe /tmp/context/grpc_health_probe
for bin in ${bins[@]}; do
  VALUE="${bin##*:}"
  cp target/x86_64-unknown-linux-musl/release/$VALUE /tmp/context/
done

for bin in ${bins[@]}; do
  KEY="${bin%%:*}"
  VALUE="${bin##*:}"
  docker build /tmp/context/ -f $KEY -t registry.dev.cheetah.games/cheetah/platform/${VALUE}:${version}
done

for bin in ${bins[@]}; do
  VALUE="${bin##*:}"
  docker push registry.dev.cheetah.games/cheetah/platform/${VALUE}:${version}
done
