# определяем зависимости для кеширования
FROM akviring/rust-builder:1.52.1-01 as planner
WORKDIR planner
COPY server/auth/cerberus/Library server/auth/cerberus/Library
COPY server/auth/cerberus/Service server/auth/cerberus/Service
COPY server/auth/cerberus/Cargo.toml server/auth/cerberus/Cargo.toml
COPY server/auth/cerberus/Cargo.lock server/auth/cerberus/Cargo.lock
RUN cd server/auth/cerberus/ && cargo chef prepare --recipe-path recipe.json

# кешируем зависимости
FROM akviring/rust-builder:1.52.1-01 as cacher
WORKDIR cache
COPY --from=planner /planner/server/auth/cerberus/recipe.json server/auth/cerberus/recipe.json
RUN cd server/auth/cerberus/ && cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

# компиляция приложения
FROM akviring/rust-builder:1.52.1-01 as builder
WORKDIR build
COPY server/auth/cerberus/Library server/auth/cerberus/Library
COPY server/auth/cerberus/Service server/auth/cerberus/Service
COPY server/auth/cerberus/Cargo.toml server/auth/cerberus/Cargo.toml
COPY server/auth/cerberus/Cargo.lock server/auth/cerberus/Cargo.lock
COPY proto/auth/cerberus proto/auth/cerberus
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/auth/cerberus/target/ server/auth/cerberus/target/
RUN touch server/auth/cerberus/Service/build.rs
RUN cd server/auth/cerberus/ && cargo build --release --target x86_64-unknown-linux-musl

# образ для запуска
FROM alpine:3.13.5
EXPOSE 5001/tcp 5002/tcp
COPY --chmod=555 --from=builder /build/server/auth/cerberus/target/x86_64-unknown-linux-musl/release/service /service
RUN adduser -D service
USER service
CMD ["/service"]
