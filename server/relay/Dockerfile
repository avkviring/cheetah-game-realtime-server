# определяем зависимости для кеширования
FROM akviring/rust-builder:1.52.1-01 as planner
WORKDIR planner
COPY server/relay/ server/relay/
RUN cd server/relay/ && cargo chef prepare --recipe-path recipe.json

# кешируем зависимости
FROM akviring/rust-builder:1.52.1-01 as cacher
WORKDIR cache
COPY --from=planner /planner/server/relay/recipe.json server/relay/recipe.json
RUN cd server/relay/ && cargo chef cook --release --recipe-path recipe.json --target=x86_64-unknown-linux-musl

# компиляция приложения
FROM akviring/rust-builder:1.52.1-01 as builder
WORKDIR build
COPY server/relay/ server/relay/
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY --from=cacher /cache/server/relay/target server/relay/target/
RUN touch server/relay/Macro/src/lib.rs
RUN cd server/relay/Server && cargo build --release --offline --target=x86_64-unknown-linux-musl


# образ для запуска
FROM alpine:3.13.5
EXPOSE 5001/tcp 5002/tcp 5555/udp 8080/tcp
COPY --chmod=555 --from=builder /build/server/relay/target/x86_64-unknown-linux-musl/release/service /service
RUN adduser -D service
USER service
CMD ["/service"]
