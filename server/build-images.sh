#!/bin/bash
set -e
export PLATFORM_VERSION=999.999.999
docker compose -f images.yaml build
docker compose -f images.yaml push