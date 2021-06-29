# определяем зависимости для кеширования
FROM akviring/rust-builder:1.52.1-01 as planner
WORKDIR planner
COPY server/cerberus/ server/cerberus/
COPY server/match/StubMatchmaking/ server/source_for_build/
RUN cd server/source_for_build/ && cargo chef prepare --recipe-path recipe.json

# собираем зависимости
FROM akviring/rust-builder:1.52.1-01 as cacher
WORKDIR cache
COPY --from=planner /planner/server/source_for_build/recipe.json server/source_for_build/recipe.json
RUN cd server/source_for_build/ && cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

# компиляция приложения
FROM akviring/rust-builder:1.52.1-01 as builder
WORKDIR build
COPY proto/cerberus proto/cerberus
COPY server/cerberus/Library server/cerberus/Library
COPY server/cerberus/Service server/cerberus/Service
COPY server/cerberus/Cargo.toml server/cerberus/Cargo.toml
COPY server/cerberus/Cargo.lock server/cerberus/Cargo.lock
COPY server/match/StubMatchmaking/Service server/source_for_build/Service
COPY server/match/StubMatchmaking/Cargo.toml server/source_for_build/Cargo.toml
COPY server/match/StubMatchmaking/Cargo.lock server/source_for_build/Cargo.lock
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/source_for_build/target/ server/source_for_build/target/
RUN touch server/source_for_build/Service/build.rs
RUN cd server/source_for_build/ && cargo build --offline --release --target x86_64-unknown-linux-musl

# образ для запуска
FROM alpine:3.13.5
EXPOSE 5001/tcp 5002/tcp
COPY --chmod=555 --from=builder /build/server/source_for_build/target/x86_64-unknown-linux-musl/release/service /service
RUN adduser -D service
USER service
CMD ["/service"]