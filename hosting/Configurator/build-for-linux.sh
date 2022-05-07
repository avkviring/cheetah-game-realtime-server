RUSTFLAGS="-C linker=x86_64-linux-musl-gcc -Ctarget-cpu=haswell -Ctarget-feature=+avx2" cargo build --release --target x86_64-unknown-linux-musl
