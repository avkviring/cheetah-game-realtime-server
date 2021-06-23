#!/bin/bash
#
# Сборка всех docker image
#
set -e
export DOCKER_BUILDKIT=1
COMPONENTS=(authentication cerberus)
for component in "${COMPONENTS[@]}"; do
  docker build ../. -f $component/Dockerfile -t docker.registry.cheetah.games/$component:999.999.999
done
for component in "${COMPONENTS[@]}"; do
  docker push docker.registry.cheetah.games/$component:999.999.999
done
