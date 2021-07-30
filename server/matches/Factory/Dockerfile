# определяем зависимости для кеширования
FROM akviring/rust-builder:1.54.0-01 as planner
WORKDIR planner
COPY server/Microservice/ server/Microservice/
COPY server/matches/Factory/ server/matches/Factory/
RUN cd server/matches/Factory/ && cargo chef prepare --recipe-path recipe.json

# собираем зависимости
FROM akviring/rust-builder:1.54.0-01 as cacher
WORKDIR cache
COPY server/Microservice/ server/Microservice/
COPY --from=planner /planner/server/matches/Factory/recipe.json server/matches/Factory/recipe.json
RUN cd server/matches/Factory/ && cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

# компиляция приложения
FROM akviring/rust-builder:1.54.0-01 as builder
WORKDIR build
COPY proto/matches/Factory/ proto/matches/Factory/
COPY proto/matches/Relay/ proto/matches/Relay/
COPY proto/matches/Registry/ proto/matches/Registry/
COPY server/Microservice/ server/Microservice/
COPY server/matches/Factory/Service server/matches/Factory/Service
COPY server/matches/Factory/Cargo.toml server/matches/Factory/Cargo.toml
COPY server/matches/Factory/Cargo.lock server/matches/Factory/Cargo.lock
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/matches/Factory/target/ server/matches/Factory/target/
RUN touch server/matches/Factory/Service/build.rs
RUN cd server/matches/Factory/ && cargo build --offline --release --target x86_64-unknown-linux-musl

# образ для запуска
FROM alpine:3.13.5
EXPOSE 5001/tcp 5002/tcp
COPY --chmod=555 --from=builder /build/server/matches/Factory/target/x86_64-unknown-linux-musl/release/service /service
RUN adduser -D service
USER service
CMD ["/service"]