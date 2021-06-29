# определяем зависимости для кеширования
FROM akviring/rust-builder:1.52.1-01 as planner
WORKDIR planner
COPY server/match/StubRegistry/ server/match/StubRegistry/
RUN cd server/match/StubRegistry/ && cargo chef prepare --recipe-path recipe.json

# собираем зависимости
FROM akviring/rust-builder:1.52.1-01 as cacher
WORKDIR cache
COPY --from=planner /planner/server/match/StubRegistry/recipe.json server/match/StubRegistry/recipe.json
RUN cd server/match/StubRegistry/ && cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

# компиляция приложения
FROM akviring/rust-builder:1.52.1-01 as builder
WORKDIR build
COPY server/match/StubRegistry/Service server/match/StubRegistry/Service
COPY server/match/StubRegistry/Cargo.toml server/match/StubRegistry/Cargo.toml
COPY server/match/StubRegistry/Cargo.lock server/match/StubRegistry/Cargo.lock
COPY proto/match/Registry/ proto/match/Registry/
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/match/StubRegistry/target/ server/match/StubRegistry/target/
RUN touch server/match/StubRegistry/Service/build.rs
RUN cd server/match/StubRegistry/ && cargo build --offline --release --target x86_64-unknown-linux-musl

# образ для запуска
FROM alpine:3.13.5
EXPOSE 5001/tcp 5002/tcp
COPY --chmod=555 --from=builder /build/server/match/StubRegistry/target/x86_64-unknown-linux-musl/release/service /service
RUN adduser -D service
USER service
CMD ["/service"]