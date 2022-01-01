# определяем зависимости для кеширования
FROM akviring/rust-builder:1.57.0-01 as planner
ARG RUSTFLAGS_ARG
WORKDIR planner
COPY server/Microservice/ server/Microservice/
COPY server/auth/Cerberus/ server/auth/Cerberus/
RUN cd server/auth/Cerberus/ && RUSTFLAGS="$RUSTFLAGS_ARG" cargo chef prepare --recipe-path recipe.json

# кешируем зависимости
FROM akviring/rust-builder:1.57.0-01 as cacher
ARG RUSTFLAGS_ARG
WORKDIR cache
COPY server/Microservice/ server/Microservice/
COPY --from=planner /planner/server/auth/Cerberus/recipe.json server/auth/Cerberus/recipe.json
RUN cd server/auth/Cerberus/ && RUSTFLAGS="$RUSTFLAGS_ARG" cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

# компиляция приложения
FROM akviring/rust-builder:1.57.0-01 as builder
ARG RUSTFLAGS_ARG
WORKDIR build
COPY server/Microservice/ server/Microservice/
COPY server/auth/Cerberus/ server/auth/Cerberus/
COPY proto/auth/Cerberus proto/auth/Cerberus
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/auth/Cerberus/target/ server/auth/Cerberus/target/
RUN touch server/auth/Cerberus/build.rs
RUN cd server/auth/Cerberus/ && RUSTFLAGS="$RUSTFLAGS_ARG" cargo build --release --target x86_64-unknown-linux-musl

# образ для запуска
FROM alpine:3.13.5
EXPOSE 5001/tcp 5002/tcp
COPY --chmod=555 --from=builder /build/server/auth/Cerberus/target/x86_64-unknown-linux-musl/release/cheetah-auth-cerberus /service
RUN adduser -D service
USER service
CMD ["/service"]
