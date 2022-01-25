# определяем зависимости для кеширования
FROM akviring/rust-builder:1.57.0-01 as planner
ARG RUSTFLAGS_ARG
WORKDIR planner
COPY server/Cargo.lock server/matches/Registry/
COPY server/Microservice/ server/Microservice/
COPY server/matches/Registry/ server/matches/Registry/
RUN cd server/matches/Registry/ && RUSTFLAGS="$RUSTFLAGS_ARG" cargo chef prepare --recipe-path recipe.json

# собираем зависимости
FROM akviring/rust-builder:1.57.0-01 as cacher
ARG RUSTFLAGS_ARG
WORKDIR cache
COPY server/Microservice/ server/Microservice/
COPY --from=planner /planner/server/matches/Registry/recipe.json server/matches/Registry/recipe.json
RUN cd server/matches/Registry/ && RUSTFLAGS="$RUSTFLAGS_ARG" cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

# компиляция приложения
FROM akviring/rust-builder:1.57.0-01 as builder
ARG RUSTFLAGS_ARG
WORKDIR build
COPY server/Cargo.lock server/matches/Registry/
COPY server/Microservice/ server/Microservice/
COPY server/matches/Registry server/matches/Registry
COPY proto/matches/Registry/ proto/matches/Registry/
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/matches/Registry/target/ server/matches/Registry/target/
RUN touch server/matches/Registry/build.rs
RUN cd server/matches/Registry/ && RUSTFLAGS="$RUSTFLAGS_ARG" cargo build --offline --release --target x86_64-unknown-linux-musl

# образ для запуска
FROM alpine:3.13.5
EXPOSE 5001/tcp 5002/tcp
COPY --chmod=555 --from=builder /build/server/matches/Registry/target/x86_64-unknown-linux-musl/release/cheetah-matches-registry /service
RUN adduser -D service
USER service
CMD ["/service"]