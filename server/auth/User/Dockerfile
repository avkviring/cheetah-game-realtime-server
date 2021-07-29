# определяем зависимости для кеширования
FROM akviring/rust-builder:1.52.1-01 as planner
WORKDIR planner
COPY server/auth/Cerberus/ server/auth/Cerberus/
COPY server/Microservice server/Microservice
COPY server/auth/User/ server/auth/User/
RUN cd server/auth/User/ && cargo chef prepare --recipe-path recipe.json

# собираем зависимости
FROM akviring/rust-builder:1.52.1-01 as cacher
WORKDIR cache
COPY proto/auth/ proto/auth/
COPY server/Microservice server/Microservice
COPY server/auth/Cerberus/Service server/auth/Cerberus/Service
COPY --from=planner /planner/server/auth/User/recipe.json server/auth/User/recipe.json
RUN cd server/auth/User/ && cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

# компиляция приложения
FROM akviring/rust-builder:1.52.1-01 as builder
WORKDIR build
COPY proto/auth/ proto/auth/
COPY server/Microservice server/Microservice
COPY server/auth/Cerberus/Service server/auth/Cerberus/Service
COPY server/auth/Cerberus/Cargo.toml server/auth/Cerberus/Cargo.toml
COPY server/auth/Cerberus/Cargo.lock server/auth/Cerberus/Cargo.lock

# копируем исходники
COPY server/auth/User/migrations server/auth/User/migrations
COPY server/auth/User/src server/auth/User/src
COPY server/auth/User/Cargo.toml server/auth/User/Cargo.toml
COPY server/auth/User/Cargo.lock server/auth/User/Cargo.lock

COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/auth/User/target/ server/auth/User/target/
RUN cd server/auth/User/ && cargo build --offline --release --target x86_64-unknown-linux-musl

# образ для запуска
FROM alpine:3.13.5
EXPOSE 5001/tcp 5002/tcp
COPY --chmod=555 --from=builder /build/server/auth/User/target/x86_64-unknown-linux-musl/release/service /service
RUN adduser -D service
USER service
CMD ["/service"]