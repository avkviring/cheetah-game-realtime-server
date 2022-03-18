1FROM registry.dev.cheetah.games/cheetah/rust-builder-image:1.0.4 as builder
ARG RUSTFLAGS_ARG
COPY . .
RUN cd server/ && cargo fetcher --url file:///root/crates/ --include-index sync
RUN cd server/Accounts/ && RUSTFLAGS="$RUSTFLAGS_ARG" cargo build --offline --release --target x86_64-unknown-linux-musl

# образ для запуска
FROM alpine:3.13.5
EXPOSE 5000/tcp
COPY server/grpc_health_probe /bin/grpc_health_probe
RUN chmod +x /bin/grpc_health_probe
COPY --from=builder /build/server/Accounts/target/x86_64-unknown-linux-musl/release/cheetah-accounts /service
RUN adduser -D service
USER service
CMD ["/service"]


# определяем зависимости для кеширования
#FROM registry.dev.cheetah.games/cheetah/rust-builder-image:1.0.4 as planner
#ARG RUSTFLAGS_ARG
#WORKDIR planner
#COPY server/Cargo.lock server/Accounts/
#COPY server/Microservice/ server/Microservice/
#COPY server/Accounts/ server/Accounts/
#RUN cd server/Accounts/ && RUSTFLAGS="$RUSTFLAGS_ARG" cargo chef prepare --recipe-path recipe.json
## кешируем зависимости
#FROM registry.dev.cheetah.games/cheetah/rust-builder-image:1.0.4 as cacher
#ARG RUSTFLAGS_ARG
#WORKDIR cache
#COPY server/Microservice/ server/Microservice/
#COPY --from=planner /planner/server/Accounts/recipe.json server/Accounts/recipe.json
#RUN cd server/Accounts/ && RUSTFLAGS="$RUSTFLAGS_ARG" cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl
#
## компиляция приложения
#FROM registry.dev.cheetah.games/cheetah/rust-builder-image:1.0.4 as builder
#ARG RUSTFLAGS_ARG
#WORKDIR build
#COPY server/Cargo.lock server/Accounts/
#COPY server/Microservice/ server/Microservice/
#COPY server/Accounts/ server/Accounts/
#COPY proto/Accounts/ proto/Accounts/
#COPY --from=cacher $CARGO_HOME $CARGO_HOME
#COPY --from=cacher /cache/server/Accounts/target/ server/Accounts/target/
#RUN touch server/Accounts/build.rs
#RUN cd server/Accounts/ && RUSTFLAGS="$RUSTFLAGS_ARG" cargo build --release --target x86_64-unknown-linux-musl
#
