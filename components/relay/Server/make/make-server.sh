#!/bin/bash

cd ../../
cross build --manifest-path Server/Cargo.toml --target x86_64-unknown-linux-gnu --release --message-format short
rsync -v target/x86_64-unknown-linux-gnu/release/cheetah_relay root@167.172.138.215:/relay/server
rsync -v target/x86_64-unknown-linux-gnu/release/cheetah_relay root@178.62.194.212:/relay/server
ssh root@167.172.138.215 -C pkill -9 server
ssh root@178.62.194.212 -C pkill -9 server
sleep 2
curl -Is http://167.172.138.215:8080/dump |head -n 1
curl -Is http://167.172.138.212:8080/dump |head -n 1