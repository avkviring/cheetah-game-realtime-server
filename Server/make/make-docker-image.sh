VERSION=`echo $1 | sed "s/refs\/tags\///g"`
cd ../
cargo clean
cd ../
cargo build --manifest-path Server/Cargo.toml --target x86_64-unknown-linux-gnu --release --message-format short
cp Server/target/x86_64-unknown-linux-gnu/release/cheetah_relay Server/docker/
cd Server/docker
docker build . -t docker.pkg.github.com/avkviring/server-relay/server:$VERSION
docker push docker.pkg.github.com/avkviring/server-relay/server:$VERSION
rm cheetah_relay