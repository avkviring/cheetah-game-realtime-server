# определяем зависимости для кеширования
FROM akviring/rust-builder:1.57.0-01 as planner
ARG RUSTFLAGS_ARG
WORKDIR planner
COPY server/Cargo.lock server/matches/Relay/Server/
COPY server/Microservice/ server/Microservice/
COPY server/matches/Relay/ server/matches/Relay/
RUN cd server/matches/Relay/Server && RUSTFLAGS="$RUSTFLAGS_ARG" cargo chef prepare --recipe-path recipe.json

# кешируем зависимости
FROM akviring/rust-builder:1.57.0-01 as cacher
ARG RUSTFLAGS_ARG
WORKDIR cache
COPY server/Microservice/ server/Microservice/
COPY server/matches/Relay/Common/ server/matches/Relay/Common/
COPY server/matches/Relay/Macro/ server/matches/Relay/Macro/
COPY --from=planner /planner/server/matches/Relay/Server/recipe.json server/matches/Relay/Server/recipe.json
RUN cd server/matches/Relay/Server && RUSTFLAGS="$RUSTFLAGS_ARG" cargo chef cook --release --recipe-path recipe.json --target=x86_64-unknown-linux-musl

# компиляция приложения
FROM akviring/rust-builder:1.57.0-01 as builder
ARG RUSTFLAGS_ARG
WORKDIR build
COPY server/Microservice/ server/Microservice/
COPY proto/matches/Relay/ proto/matches/Relay/
COPY server/Cargo.lock server/matches/Relay/Server/
COPY server/matches/Relay/Common/ server/matches/Relay/Common/
COPY server/matches/Relay/Macro/ server/matches/Relay/Macro/
COPY server/matches/Relay/Client/ server/matches/Relay/Client/
COPY server/matches/Relay/Server/ server/matches/Relay/Server/
COPY server/Cargo.lock server/matches/Relay/Server/
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/matches/Relay/Server/target server/matches/Relay/Server/target/
RUN touch server/matches/Relay/Macro/src/lib.rs
RUN touch server/matches/Relay/Server/build.rs
RUN cd server/matches/Relay/Server/ && RUSTFLAGS="$RUSTFLAGS_ARG" cargo build --release --offline --target=x86_64-unknown-linux-musl


# образ для запуска
FROM alpine:3.13.5
# публичный игровой порт к которому подключаются клиенты
EXPOSE 5555/udp
COPY --chmod=555 --from=builder /build/server/matches/Relay/Server/target/x86_64-unknown-linux-musl/release/service /service
RUN adduser -D service
USER service
CMD ["/service"]
