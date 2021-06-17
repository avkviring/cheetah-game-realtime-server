# определяем зависимости для кеширования
FROM akviring/rust-builder:1.52.1-01 as planner
WORKDIR planner
COPY server/cerberus/Library server/cerberus/Library
COPY server/cerberus/Service server/cerberus/Service
COPY server/cerberus/Cargo.toml server/cerberus/Cargo.toml
COPY server/cerberus/Cargo.lock server/cerberus/Cargo.lock
RUN cd server/cerberus/ && cargo chef prepare --recipe-path recipe.json

# кешируем зависимости
FROM akviring/rust-builder:1.52.1-01 as cacher
WORKDIR cache
COPY --from=planner /planner/server/cerberus/recipe.json server/cerberus/recipe.json
RUN cd server/cerberus/ && cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

# компиляция приложения
FROM akviring/rust-builder:1.52.1-01 as builder
WORKDIR build
COPY server/cerberus/Library server/cerberus/Library
COPY server/cerberus/Service server/cerberus/Service
COPY server/cerberus/Cargo.toml server/cerberus/Cargo.toml
COPY server/cerberus/Cargo.lock server/cerberus/Cargo.lock
COPY proto/cerberus proto/cerberus
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/cerberus/target/ server/cerberus/target/
RUN touch server/cerberus/Service/build.rs
RUN cd server/cerberus/ && cargo build --release --target x86_64-unknown-linux-musl

# образ для запуска
FROM alpine:3.13.5
EXPOSE 5001/tcp 5002/tcp
COPY --chmod=555 --from=builder /build/server/cerberus/target/x86_64-unknown-linux-musl/release/service /service
RUN adduser -D service
USER service
CMD ["/service"]
