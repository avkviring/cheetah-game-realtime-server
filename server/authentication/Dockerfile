# определяем зависимости для кеширования
FROM akviring/rust-builder:1.52.1-01 as planner
WORKDIR planner
COPY server/cerberus/ server/cerberus/
COPY server/authentication/ server/authentication/
RUN cd server/authentication/ && cargo chef prepare --recipe-path recipe.json

# собираем зависимости
FROM akviring/rust-builder:1.52.1-01 as cacher
WORKDIR cache
COPY server/cerberus/Library server/cerberus/Library
COPY server/cerberus/Service server/cerberus/Service
COPY --from=planner /planner/server/authentication/recipe.json server/authentication/recipe.json
RUN cd server/authentication/ && cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

# компиляция приложения
FROM akviring/rust-builder:1.52.1-01 as builder
WORKDIR build
COPY proto/cerberus proto/cerberus
COPY proto/authentication proto/authentication
COPY server/cerberus/Library server/cerberus/Library
COPY server/cerberus/Service server/cerberus/Service
COPY server/cerberus/Cargo.toml server/cerberus/Cargo.toml
COPY server/cerberus/Cargo.lock server/cerberus/Cargo.lock
COPY server/authentication/Service server/authentication/Service
COPY server/authentication/Cargo.toml server/authentication/Cargo.toml
COPY server/authentication/Cargo.lock server/authentication/Cargo.lock
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/authentication/target/ server/authentication/target/
RUN touch server/authentication/Service/build.rs
RUN cd server/authentication/ && cargo build --offline --release --target x86_64-unknown-linux-musl

# образ для запуска
FROM alpine:3.13.5
EXPOSE 5001/tcp 5002/tcp
COPY --chmod=555 --from=builder /build/server/authentication/target/x86_64-unknown-linux-musl/release/service /service
RUN adduser -D service
USER service
CMD ["/service"]