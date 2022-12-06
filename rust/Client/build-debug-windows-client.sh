#!/bin/bash
## сборка runtime клиента для тестирования на macos
OUTPUT="../../../../../../client/Unity/Packages/games.cheetah.matches.realtime/"
rm -f $OUTPUT/x86_64/windows.dll
cargo build
cp ../../../../../target/debug/cheetah_client.dll "$OUTPUT/Runtime/Library/windows.dll"
cp ../../../../../target/debug/cheetah_client.pdb "$OUTPUT/Runtime/Library/windows.pdb"
