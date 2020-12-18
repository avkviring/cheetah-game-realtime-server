#!/bin/bash

cd ../../
cross build --manifest-path Server/Cargo.toml --target x86_64-unknown-linux-gnu --release --message-format short
scp  target/x86_64-unknown-linux-gnu/release/cheetah_relay root@167.172.138.215:/relay/server