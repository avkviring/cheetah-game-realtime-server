# определяем зависимости для кеширования
FROM akviring/rust-builder:1.52.1-01 as planner
WORKDIR planner
COPY server/auth/Cerberus/ server/auth/Cerberus/
COPY server/Microservice server/Microservice
COPY server/auth/Authentication/ server/auth/Authentication/
RUN cd server/auth/Authentication/ && cargo chef prepare --recipe-path recipe.json

# собираем зависимости
FROM akviring/rust-builder:1.52.1-01 as cacher
WORKDIR cache
COPY server/Microservice server/Microservice
COPY server/auth/Cerberus/Service server/auth/Cerberus/Service
COPY --from=planner /planner/server/auth/Authentication/recipe.json server/auth/Authentication/recipe.json
RUN cd server/auth/Authentication/ && cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

# компиляция приложения
FROM akviring/rust-builder:1.52.1-01 as builder
WORKDIR build
COPY proto/auth/Cerberus proto/auth/Cerberus
COPY proto/auth/Cookie/ proto/auth/Cookie/
COPY proto/auth/Google/ proto/auth/Google/
COPY server/Microservice server/Microservice
COPY server/auth/Cerberus/Service server/auth/Cerberus/Service
COPY server/auth/Cerberus/Cargo.toml server/auth/Cerberus/Cargo.toml
COPY server/auth/Cerberus/Cargo.lock server/auth/Cerberus/Cargo.lock
COPY server/auth/Authentication/Service server/auth/Authentication/Service
COPY server/auth/Authentication/Cargo.toml server/auth/Authentication/Cargo.toml
COPY server/auth/Authentication/Cargo.lock server/auth/Authentication/Cargo.lock
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/auth/Authentication/target/ server/auth/Authentication/target/
RUN touch server/auth/Authentication/Service/build.rs
RUN cd server/auth/Authentication/ && cargo build --offline --release --target x86_64-unknown-linux-musl

# образ для запуска
FROM alpine:3.13.5
EXPOSE 5001/tcp 5002/tcp
COPY --chmod=555 --from=builder /build/server/auth/Authentication/target/x86_64-unknown-linux-musl/release/service /service
RUN adduser -D service
USER service
CMD ["/service"]