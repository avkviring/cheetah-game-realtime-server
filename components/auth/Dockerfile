# 1: Build the exe
FROM rust:1.50 as builder
WORKDIR /usr/src

# 1a: Prepare for static linking
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y musl-tools && \
    apt-get install libssl-dev && \
    apt-get install musl-tools && \
    rustup target add x86_64-unknown-linux-musl

RUN rustup component add rustfmt

# build openssl
RUN ln -s /usr/include/x86_64-linux-gnu/asm /usr/include/x86_64-linux-musl/asm && \
    ln -s /usr/include/asm-generic /usr/include/x86_64-linux-musl/asm-generic && \
    ln -s /usr/include/linux /usr/include/x86_64-linux-musl/linux &&\
    mkdir /musl &&\
    wget https://github.com/openssl/openssl/archive/OpenSSL_1_1_1f.tar.gz &&\
    tar zxvf OpenSSL_1_1_1f.tar.gz &&\
    cd openssl-OpenSSL_1_1_1f/ &&\
    CC="musl-gcc -fPIE -pie" ./Configure no-shared no-async --prefix=/musl --openssldir=/musl/ssl linux-x86_64 &&\
    make depend &&\
    make -j$(nproc) &&\
    make install

# 1b: Download and compile Rust dependencies (and store as a separate Docker layer)
RUN mkdir /usr/src/service/
WORKDIR /usr/src/service/
COPY . .
RUN export PKG_CONFIG_ALLOW_CROSS=1 &&\
    export OPENSSL_STATIC=true &&\
    export OPENSSL_DIR=/musl &&\
    cargo install --path auth/server/Service/ --target x86_64-unknown-linux-musl



# 2: Copy the exe and extra files ("static") to an empty Docker image
FROM scratch
EXPOSE 5001/tcp
COPY --from=builder /usr/local/cargo/bin/games-cheetah-auth-service /service
CMD ["/service"]