#/bin/bash

# macOS prepare
if [[ ($OSTYPE == 'darwin') && !(-f /usr/local/bin/musl-gcc) ]]; then
    brew install filosottile/musl-cross/musl-cross
    ln -s /usr/local/opt/musl-cross/bin/x86_64-linux-musl-gcc /usr/local/bin/musl-gcc
fi

set -e
cd modules/
RUSTFLAGS="-C linker=x86_64-linux-musl-gcc -Ctarget-cpu=haswell -Ctarget-feature=+avx2" \
    cargo build --release --target x86_64-unknown-linux-musl
cd ../
version="999.999.999" scripts/package-server-images.sh
