# определяем зависимости для кеширования
FROM akviring/rust-builder:1.52.1-01 as planner
WORKDIR planner
COPY server/auth/Cerberus/ server/auth/Cerberus/
COPY server/Microservice server/Microservice
COPY server/auth/User/ server/auth/User/
COPY server/auth/Cookie/ server/auth/Cookie/
RUN cd server/auth/Cookie/ && cargo chef prepare --recipe-path recipe.json

# собираем зависимости
FROM akviring/rust-builder:1.52.1-01 as cacher
WORKDIR cache
COPY server/Microservice server/Microservice
COPY server/auth/Cerberus/Service server/auth/Cerberus/Service
COPY --from=planner /planner/server/auth/Cookie/recipe.json server/auth/Cookie/recipe.json
RUN cd server/auth/User/ && cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

# компиляция приложения
FROM akviring/rust-builder:1.52.1-01 as builder
WORKDIR build
COPY proto/auth/Cerberus proto/auth/Cerberus
COPY proto/auth/User proto/auth/User
COPY proto/auth/Cookie proto/auth/Cookie
COPY server/Microservice server/Microservice
COPY server/auth/Cerberus/Service server/auth/Cerberus/Service
COPY server/auth/Cerberus/Cargo.toml server/auth/Cerberus/Cargo.toml
COPY server/auth/Cerberus/Cargo.lock server/auth/Cerberus/Cargo.lock

# копируем исходники
COPY server/auth/Cookie/migrations server/auth/Cookie/migrations
COPY server/auth/Cookie/src server/auth/Cookie/src
COPY server/auth/Cookie/build.rs server/auth/Cookie/build.rs
COPY server/auth/Cookie/Cargo.toml server/auth/Cookie/Cargo.toml
COPY server/auth/Cookie/Cargo.lock server/auth/Cookie/Cargo.lock

COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/auth/Cookie/target/ server/auth/Cookie/target/
RUN touch server/auth/Cookie/build.rs
RUN cd server/auth/Cookie/ && cargo build --offline --release --target x86_64-unknown-linux-musl

# образ для запуска
FROM alpine:3.13.5
EXPOSE 5001/tcp 5002/tcp
COPY --chmod=555 --from=builder /build/server/auth/Cookie/target/x86_64-unknown-linux-musl/release/service /service
RUN adduser -D service
USER service
CMD ["/service"]