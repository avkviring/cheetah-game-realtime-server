#!/bin/bash
set -e
export COMPOSE_DOCKER_CLI_BUILD=1
export DOCKER_BUILDKIT=1
docker-compose up --build --abort-on-container-exit --remove-orphans

