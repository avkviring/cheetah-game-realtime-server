# определяем зависимости для кеширования
FROM rust:1.50 as planner
RUN cargo install cargo-chef
WORKDIR app
COPY server/relay/Macro server/relay/Macro
COPY server/relay/Common server/relay/Common
COPY server/relay/Server server/relay/Server
RUN cd server/relay/Server && cargo chef prepare --recipe-path recipe.json

# кешируем зависимости
FROM rust:1.50 as cacher
RUN cargo install cargo-chef
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl
WORKDIR app
COPY server/relay/Macro server/relay/Macro
COPY server/relay/Common server/relay/Common
COPY server/relay/Server server/relay/Server
COPY --from=planner /app/server/relay/Server/recipe.json server/relay/Server/recipe.json
RUN cd server/relay/Server && cargo chef cook --release --recipe-path recipe.json

# компиляция приложения
FROM rust:1.50 as builder
WORKDIR app
RUN rustup target add x86_64-unknown-linux-musl
RUN rustup component add rustfmt
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /app/server/relay/Server/target server/relay/Server/target/
COPY server/relay/Macro server/relay/Macro
COPY server/relay/Common server/relay/Common
COPY server/relay/Server server/relay/Server
RUN cargo build --release --offline --manifest-path server/relay/Server/Cargo.toml

# образ для запуска
FROM rust:1.50
EXPOSE 5001/tcp 5002/tcp
COPY --from=builder /app/server/relay/Server/target/release/cheetah_relay /service
CMD ["/service"]