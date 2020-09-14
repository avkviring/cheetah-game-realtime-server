#!/bin/bash

cd ../../
# macos
cargo build --manifest-path Server/Cargo.toml  --release --message-format short

# windows
cross build --manifest-path Server/Cargo.toml --target x86_64-pc-windows-gnu --release --message-format short
