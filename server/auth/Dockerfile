# определяем зависимости для кеширования
FROM rust:1.50 as planner
RUN cargo install cargo-chef
WORKDIR app
COPY server/cerberus server/cerberus
COPY server/auth server/auth
RUN cd server/auth && cargo chef prepare --recipe-path recipe.json

# кешируем зависимости
FROM rust:1.50 as cacher
RUN cargo install cargo-chef
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl
WORKDIR app
COPY server/cerberus /cerberus
COPY proto/cerberus proto/cerberus
COPY server/auth server/auth
COPY proto/auth proto/auth
COPY --from=planner /app/server/auth/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json



# компиляция приложения
FROM rust:1.50 as builder
WORKDIR app
RUN rustup target add x86_64-unknown-linux-musl
RUN rustup component add rustfmt
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /app/target server/auth/target/
COPY server/cerberus server/cerberus
COPY proto/cerberus proto/cerberus
COPY server/auth server/auth
COPY proto/auth proto/auth
RUN cargo build --release --offline --manifest-path server/auth/Service/Cargo.toml

# образ для запуска
FROM rust:1.50
EXPOSE 5001/tcp 5002/tcp
COPY --from=builder /app/server/auth/target/release/games-cheetah-auth-service /service
CMD ["/service"]