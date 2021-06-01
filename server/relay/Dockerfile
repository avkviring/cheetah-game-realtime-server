# определяем зависимости для кеширования
FROM rust:1.50 as planner
RUN cargo install cargo-chef
WORKDIR planner
COPY server/relay/ server/relay/
RUN cd server/relay/ && cargo chef prepare --recipe-path recipe.json

# кешируем зависимости
FROM rust:1.50 as cacher
RUN cargo install cargo-chef
RUN apt-get update && \
    apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl
WORKDIR cache
COPY --from=planner /planner/server/relay/recipe.json server/relay/recipe.json
RUN cd server/relay/ && cargo chef cook --release --recipe-path recipe.json --target=x86_64-unknown-linux-musl

# компиляция приложения
FROM rust:1.50 as builder
WORKDIR build
RUN cargo install cargo-chef
RUN apt-get update && \
    apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl
COPY server/relay/ server/relay/
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/relay/target server/relay/target/
RUN touch server/relay/Macro/src/lib.rs
RUN cd server/relay/Server && cargo build --release --offline --target=x86_64-unknown-linux-musl


# образ для запуска
FROM alpine:3.13.5
EXPOSE 5001/tcp 5002/tcp 5555/udp 8080/tcp
COPY --chmod=555 --from=builder /build/server/relay/target/x86_64-unknown-linux-musl/release/service /service
RUN adduser -D service
USER service
CMD ["/service"]
