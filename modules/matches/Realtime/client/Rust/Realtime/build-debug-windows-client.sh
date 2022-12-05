#!/bin/bash
## сборка runtime клиента для тестирования на macos
OUTPUT="../../../../../../client/Unity/Packages/games.cheetah.matches.realtime/"
rm -f $OUTPUT/x86_64/windows.dll
cargo build
cp ../../../../../target/debug/cheetah_matches_realtime_client.dll "$OUTPUT/Runtime/Library/windows.dll"
cp ../../../../../target/debug/cheetah_matches_realtime_client.pdb "$OUTPUT/Runtime/Library/windows.pdb"
