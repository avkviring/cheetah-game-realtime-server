# определяем зависимости для кеширования
FROM rust:1.50 as planner
RUN cargo install cargo-chef
WORKDIR app
COPY cerberus/server/ cerberus/server/
COPY cerberus/proto/ cerberus/proto/
COPY auth auth
RUN cd auth/server/ && cargo chef prepare --recipe-path recipe.json

# кешируем зависимости
FROM rust:1.50 as cacher
RUN cargo install cargo-chef
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl
WORKDIR app
COPY cerberus/server/ /cerberus/server/
COPY --from=planner /app/auth/server/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json



# компиляция приложения
FROM rust:1.50 as builder
WORKDIR app
RUN rustup target add x86_64-unknown-linux-musl
RUN rustup component add rustfmt
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /app/target auth/server/target/
COPY cerberus/proto/ cerberus/proto/
COPY cerberus/server/ cerberus/server/
COPY auth auth
RUN cargo build --release --offline --manifest-path auth/server/Service/Cargo.toml

# образ для запуска
FROM rust:1.50
EXPOSE 5001/tcp 5002/tcp
COPY --from=builder /app/auth/server/target/release/games-cheetah-auth-service /service
CMD ["/service"]