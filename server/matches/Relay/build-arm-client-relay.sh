#!/bin/bash
OUTPUT="../../../clients/Unity/Packages/games.cheetah.matches.relay/"
#rm -f $OUTPUT/Android/armv7/libcheetah_matches_relay_client.so
cross build --manifest-path matches/Relay/Client/Cargo.toml  --target armv7-linux-androideabi --release
cp ../../target/armv7-linux-androideabi/release/libcheetah_matches_relay_client.so "$OUTPUT/Android/armv7/libcheetah_matches_relay_client.so"