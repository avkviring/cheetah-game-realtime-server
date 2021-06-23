# определяем зависимости для кеширования
FROM akviring/rust-builder:1.52.1-01 as planner
WORKDIR planner
COPY server/match/Relay/ server/match/Relay/
RUN cd server/match/Relay/ && cargo chef prepare --recipe-path recipe.json

# кешируем зависимости
FROM akviring/rust-builder:1.52.1-01 as cacher
WORKDIR cache
COPY --from=planner /planner/server/match/Relay/recipe.json server/match/Relay/recipe.json
RUN cd server/match/Relay/ && cargo chef cook --release --recipe-path recipe.json --target=x86_64-unknown-linux-musl

# компиляция приложения
FROM akviring/rust-builder:1.52.1-01 as builder
WORKDIR build
COPY proto/match/Relay/ proto/match/Relay/
COPY server/match/Relay/Common/ server/match/Relay/Common/
COPY server/match/Relay/Macro/ server/match/Relay/Macro/
COPY server/match/Relay/Client/ server/match/Relay/Client/
COPY server/match/Relay/Server/ server/match/Relay/Server/
COPY server/match/Relay/Cargo.toml server/match/Relay/Cargo.toml
COPY server/match/Relay/Cargo.lock server/match/Relay/Cargo.lock
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/match/Relay/target server/match/Relay/target/
RUN touch server/match/Relay/Macro/src/lib.rs
RUN touch server/match/Relay/Server/build.rs
RUN cd server/match/Relay/Server/ && cargo build --release --offline --target=x86_64-unknown-linux-musl


# образ для запуска
FROM alpine:3.13.5
EXPOSE 5001/tcp 5002/tcp 5555/udp 8080/tcp
COPY --chmod=555 --from=builder /build/server/match/Relay/target/x86_64-unknown-linux-musl/release/service /service
RUN adduser -D service
USER service
CMD ["/service"]
