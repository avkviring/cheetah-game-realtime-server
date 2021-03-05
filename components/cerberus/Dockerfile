# 1: Build the exe
FROM rust:1.50 as builder
WORKDIR /usr/src

# 1a: Prepare for static linking
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl

RUN rustup component add rustfmt

# 1b: Download and compile Rust dependencies (and store as a separate Docker layer)
RUN mkdir /usr/src/service/
WORKDIR /usr/src/service/
COPY . .
RUN cargo install --path cerberus/server/Service/ --target x86_64-unknown-linux-musl

# 2: Copy the exe and extra files ("static") to an empty Docker image
FROM scratch
EXPOSE 5001/tcp 5002/tcp
COPY --from=builder /usr/local/cargo/bin/games-cheetah-cerberus-service /service
CMD ["/service"]