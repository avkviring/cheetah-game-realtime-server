#/bin/bash
set -e
export PLATFORM_VERSION=999.999.999
docker compose -f images.yaml build --build-arg RUSTFLAGS_ARG="-Ctarget-cpu=haswell -Ctarget-feature=+avx2"
docker compose -f images.yaml push