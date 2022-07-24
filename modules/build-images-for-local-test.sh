#/bin/bash
set -e
RUSTFLAGS="-C linker=x86_64-linux-musl-gcc -Ctarget-cpu=haswell -Ctarget-feature=+avx2" cargo build --release --target x86_64-unknown-linux-musl
version="999.999.997" ./package-server-images.sh

# mac os prepare
# brew install filosottile/musl-cross/musl-cross
# ln -s /usr/local/opt/musl-cross/bin/x86_64-linux-musl-gcc /usr/local/bin/musl-gcc
