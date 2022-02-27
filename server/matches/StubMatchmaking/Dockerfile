# определяем зависимости для кеширования
FROM akviring/rust-builder:1.59.0-05 as planner
ARG RUSTFLAGS_ARG
WORKDIR planner
COPY server/Cargo.lock server/matches/StubMatchmaking/
COPY server/Microservice/ server/Microservice/
COPY server/matches/StubMatchmaking/ server/matches/StubMatchmaking/
RUN cd server/matches/StubMatchmaking/ && RUSTFLAGS="$RUSTFLAGS_ARG" cargo chef prepare --recipe-path recipe.json

# собираем зависимости
FROM akviring/rust-builder:1.59.0-05 as cacher
ARG RUSTFLAGS_ARG
WORKDIR cache
COPY server/Microservice/ server/Microservice/
COPY --from=planner /planner/server/matches/StubMatchmaking/recipe.json server/matches/StubMatchmaking/recipe.json
RUN cd server/matches/StubMatchmaking/ && RUSTFLAGS="$RUSTFLAGS_ARG" cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

# компиляция приложения
FROM akviring/rust-builder:1.59.0-05 as builder
ARG RUSTFLAGS_ARG
WORKDIR build
COPY proto/matches/Matchmaking/ proto/matches/Matchmaking/
COPY proto/matches/Factory/ proto/matches/Factory/
COPY proto/matches/Relay/ proto/matches/Relay/
COPY proto/matches/Registry/ proto/matches/Registry/
COPY server/Cargo.lock server/matches/StubMatchmaking/
COPY server/Microservice/ server/Microservice/
COPY server/matches/StubMatchmaking server/matches/StubMatchmaking
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/matches/StubMatchmaking/target/ server/matches/StubMatchmaking/target/
RUN touch server/matches/StubMatchmaking/build.rs
RUN cd server/matches/StubMatchmaking/ && RUSTFLAGS="$RUSTFLAGS_ARG" cargo build --offline --release --target x86_64-unknown-linux-musl

# образ для запуска
FROM alpine:3.13.5
EXPOSE 5001/tcp 5002/tcp
COPY --chmod=555 --from=builder /build/server/matches/StubMatchmaking/target/x86_64-unknown-linux-musl/release/cheetah-matches-stub-matchmaking /service
RUN adduser -D service
USER service
CMD ["/service"]