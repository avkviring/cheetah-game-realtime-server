# определяем зависимости для кеширования
FROM akviring/rust-builder:1.57.0-01 as planner
ARG RUSTFLAGS_ARG
WORKDIR planner
COPY server/Cargo.lock server/auth/Cookie/
COPY server/auth/Cerberus/ server/auth/Cerberus/
COPY server/Microservice server/Microservice
COPY server/auth/User/ server/auth/User/
COPY server/auth/Cookie/ server/auth/Cookie/
RUN cd server/auth/Cookie/ && RUSTFLAGS="$RUSTFLAGS_ARG" cargo chef prepare --recipe-path recipe.json

# собираем зависимости
FROM akviring/rust-builder:1.57.0-01 as cacher
ARG RUSTFLAGS_ARG
WORKDIR cache
COPY proto/auth/ proto/auth/
COPY server/Microservice server/Microservice
COPY server/auth/User/ server/auth/User/
COPY server/auth/Cerberus/ server/auth/Cerberus/
COPY --from=planner /planner/server/auth/Cookie/recipe.json server/auth/Cookie/recipe.json
RUN cd server/auth/Cookie/ && RUSTFLAGS="$RUSTFLAGS_ARG" cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

# компиляция приложения
FROM akviring/rust-builder:1.57.0-01 as builder
ARG RUSTFLAGS_ARG
WORKDIR build
COPY proto/auth/ proto/auth/
COPY server/Microservice server/Microservice
COPY server/auth/User/ server/auth/User/
COPY server/auth/Cerberus/ server/auth/Cerberus/
COPY server/auth/Cookie server/auth/Cookie/
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/auth/Cookie/target/ server/auth/Cookie/target/
RUN touch server/auth/Cookie/build.rs
RUN cd server/auth/Cookie/ && RUSTFLAGS="$RUSTFLAGS_ARG" cargo build --offline --release --target x86_64-unknown-linux-musl

# образ для запуска
FROM alpine:3.13.5
EXPOSE 5001/tcp 5002/tcp
COPY --chmod=555 --from=builder /build/server/auth/Cookie/target/x86_64-unknown-linux-musl/release/cheetah-auth-cookie /service
RUN adduser -D service
USER service
CMD ["/service"]