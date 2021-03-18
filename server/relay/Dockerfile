# определяем зависимости для кеширования
FROM rust:1.50 as planner
RUN cargo install cargo-chef
WORKDIR app
COPY relay/Macro relay/Macro
COPY relay/Common relay/Common
COPY relay/Server relay/Server
RUN cd relay/Server && cargo chef prepare --recipe-path recipe.json

# кешируем зависимости
FROM rust:1.50 as cacher
RUN cargo install cargo-chef
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl
WORKDIR app
COPY relay/Macro relay/Macro
COPY relay/Common relay/Common
COPY --from=planner /app/relay/Server/recipe.json relay/Server/recipe.json
RUN cd relay/Server && cargo chef cook --release --recipe-path recipe.json

# компиляция приложения
FROM rust:1.50 as builder
WORKDIR app
RUN rustup target add x86_64-unknown-linux-musl
RUN rustup component add rustfmt
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /app/relay/Server/target relay/Server/target/
COPY relay/Macro relay/Macro
COPY relay/Common relay/Common
COPY relay/Server relay/Server
RUN cargo build --release --offline --manifest-path relay/Server/Cargo.toml

# образ для запуска
FROM rust:1.50
EXPOSE 5001/tcp 5002/tcp
COPY --from=builder /app/relay/Server/target/release/cheetah_relay /service
CMD ["/service"]