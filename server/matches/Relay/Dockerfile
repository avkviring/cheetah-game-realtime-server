# определяем зависимости для кеширования
FROM akviring/rust-builder:1.52.1-01 as planner
WORKDIR planner
COPY server/Microservice/ server/Microservice/
COPY server/matches/Relay/ server/matches/Relay/
RUN cd server/matches/Relay/ && cargo chef prepare --recipe-path recipe.json

# кешируем зависимости
FROM akviring/rust-builder:1.52.1-01 as cacher
WORKDIR cache
COPY server/Microservice/ server/Microservice/
COPY --from=planner /planner/server/matches/Relay/recipe.json server/matches/Relay/recipe.json
RUN cd server/matches/Relay/ && cargo chef cook --release --recipe-path recipe.json --target=x86_64-unknown-linux-musl

# компиляция приложения
FROM akviring/rust-builder:1.52.1-01 as builder
WORKDIR build
COPY server/Microservice/ server/Microservice/
COPY proto/matches/Relay/ proto/matches/Relay/
COPY server/matches/Relay/Common/ server/matches/Relay/Common/
COPY server/matches/Relay/Macro/ server/matches/Relay/Macro/
COPY server/matches/Relay/Client/ server/matches/Relay/Client/
COPY server/matches/Relay/Server/ server/matches/Relay/Server/
COPY server/matches/Relay/Cargo.toml server/matches/Relay/Cargo.toml
COPY server/matches/Relay/Cargo.lock server/matches/Relay/Cargo.lock
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/matches/Relay/target server/matches/Relay/target/
RUN touch server/matches/Relay/Macro/src/lib.rs
RUN touch server/matches/Relay/Server/build.rs
RUN cd server/matches/Relay/Server/ && cargo build --release --offline --target=x86_64-unknown-linux-musl


# образ для запуска
FROM alpine:3.13.5
EXPOSE 5001/tcp 5002/tcp 5555/udp 8080/tcp
COPY --chmod=555 --from=builder /build/server/matches/Relay/target/x86_64-unknown-linux-musl/release/service /service
RUN adduser -D service
USER service
CMD ["/service"]
